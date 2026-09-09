#!/usr/bin/env python3
"""Check npm, PyPI and Cargo installations in a disposable root through CLI/MCP."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tarfile
import tomllib

try:
    from .native_registry_packages import npm_command
except ImportError:
    from native_registry_packages import npm_command


def run(argv, *, root, env, input=None, check=True):
    return subprocess.run([str(arg) for arg in argv], cwd=root, env=env, input=input,
                          text=True, capture_output=True, check=check, timeout=120)


def check_cli(executable, *, version, root, env):
    command = executable if isinstance(executable, list) else [executable]
    assert run(command + ['--version'], root=root, env=env).stdout.strip() == f'qiongli {version}'
    assert 'qiongli project' in run(command + ['--help'], root=root, env=env).stdout
    content = run(command + ['content', 'list'], root=root, env=env)
    assert json.loads(content.stdout)
    invalid = run(command + ['not-a-command'], root=root, env=env, check=False)
    assert invalid.returncode != 0 and not invalid.stdout and 'error:' in invalid.stderr
    tools = {}
    for profile, expected_count in [('lite', 14), ('full', 32)]:
        requests = [
            {'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {}},
            {'jsonrpc': '2.0', 'id': 2, 'method': 'tools/list', 'params': {}},
            {'jsonrpc': '2.0', 'id': 3, 'method': 'tools/call', 'params': {
                'name': 'qiongli_config_status', 'arguments': {}}},
        ]
        output = run(command + ['mcp', 'serve', '--profile', profile, '--transport', 'stdio'],
                     root=root, env=env, input=''.join(json.dumps(r) + '\n' for r in requests))
        assert not output.stderr, output.stderr
        messages = {m['id']: m for m in map(json.loads, output.stdout.splitlines())}
        assert len(messages) == 3
        assert messages[1]['result']['serverInfo']['version'] == version
        names = [t['name'] for t in messages[2]['result']['tools']]
        assert len(names) == expected_count and len(set(names)) == expected_count
        assert 'error' not in messages[3] and not messages[3]['result'].get('isError', False)
        tools[profile] = len(names)
    return {'version': version, 'invalid_command_rejected': True, 'mcp_tools': tools}


def install_cargo_archives(package_root, receipt, root, env, target_dir):
    source = root / 'cargo-archives'
    source.mkdir()
    patches = []
    application = None
    for artifact in receipt['artifacts']:
        if not artifact['file'].endswith('.crate'):
            continue
        path = package_root / artifact['file']
        with tarfile.open(path) as archive:
            archive.extractall(source, filter='data')
        crate = source / path.name.removesuffix('.crate')
        package = tomllib.loads((crate / 'Cargo.toml').read_text())['package']
        assert package['version'] == receipt['version']
        if package['name'] == 'qiongli':
            application = crate
        else:
            patches.append(f'{json.dumps(package["name"])} = {{ path = {json.dumps(str(crate))} }}')
    assert application and len(patches) == 8
    config = root / 'cargo-archive-patches.toml'
    config.write_text('[patch.crates-io]\n' + '\n'.join(patches) + '\n')
    lock_path = application / 'Cargo.lock'
    sections = lock_path.read_text().split('[[package]]')
    for index, section in enumerate(sections):
        if re.search(r'^name = "qiongli-[^"]+"$', section, re.M):
            sections[index] = re.sub(r'^(?:source|checksum) = .*\n', '', section, flags=re.M)
    lock_path.write_text('[[package]]'.join(sections))
    installed = root / 'cargo-installed'
    command = ['cargo', 'install', '--path', str(application), '--bin', 'qiongli', '--bin', 'ql',
               '--root', str(installed), '--no-default-features', '--locked', '--offline',
               '--config', str(config), '--target-dir', str(target_dir)]
    # The only patches point at checksum-checked archives from this same packet.
    # This checks their source closure; it does not stand in for registry install.
    with (root / 'cargo-install.log').open('w') as log:
        subprocess.run(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT,
                       check=True, timeout=1200)
    return installed / ('bin/qiongli.exe' if os.name == 'nt' else 'bin/qiongli')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packages', type=Path)
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--cargo-archives', action='store_true', help='build only from the checked .crate archives with local dependency patches')
    parser.add_argument('--cargo-target-dir', type=Path, help='optional Cargo build cache to reuse')
    parser.add_argument('--cargo-installed', type=Path, help='optional independently installed Cargo executable')
    parser.add_argument('--cargo-only', action='store_true', help='check Cargo without npm/PyPI packages')
    parser.add_argument('--version', help='verify the expected native version')
    args = parser.parse_args()
    if args.cargo_only and not (args.cargo_archives or args.cargo_installed):
        parser.error('--cargo-only requires --cargo-archives or --cargo-installed')
    if not args.packages and not (args.cargo_only and args.cargo_installed and args.version and not args.cargo_archives):
        parser.error('--packages is required except for a versioned --cargo-only --cargo-installed check')
    package_root = args.packages.resolve() if args.packages else None
    root = args.out_dir.resolve()
    root.mkdir(parents=True, exist_ok=False)
    receipt = (json.loads((package_root / 'registry-packages.json').read_text()) if package_root
               else {'version': args.version, 'artifacts': []})
    if args.version:
        assert receipt['version'] == args.version
    paths = {}
    for artifact in receipt['artifacts']:
        path = package_root / artifact['file']
        assert path.parent == package_root and not path.is_symlink()
        assert hashlib.sha256(path.read_bytes()).hexdigest() == artifact['sha256']
        paths[path.suffix] = path
    env = os.environ.copy()
    env.setdefault('CARGO_HOME', str(Path.home() / '.cargo'))
    env.setdefault('RUSTUP_HOME', str(Path.home() / '.rustup'))
    home = root / 'home'
    home.mkdir(mode=0o700)
    env.update(HOME=str(home), USERPROFILE=str(home), QIONGLI_CONFIG_HOME=str(home / 'config'),
               PIP_DISABLE_PIP_VERSION_CHECK='1', MISE_SKIP_RESHIM='1')
    if os.name == 'nt':
        env.update(APPDATA=str(home / 'AppData/Roaming'), LOCALAPPDATA=str(home / 'AppData/Local'))
    for key in ('CODEX_HOME', 'CLAUDE_CONFIG_DIR', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    checks = {}
    if not args.cargo_only:
        node = Path(subprocess.check_output(['node', '-p', 'process.execPath'], text=True).strip())
        env['PATH'] = str(node.parent) + os.pathsep + env['PATH']
        python_root = root / 'python'
        run([sys.executable, '-m', 'venv', python_root], root=root, env=env)
        python_bin = python_root / ('Scripts' if os.name == 'nt' else 'bin')
        suffix = '.exe' if os.name == 'nt' else ''
        run([python_bin / ('python' + suffix), '-m', 'pip', 'install', '--no-index', '--no-deps', paths['.whl']], root=root, env=env)
        npm_root = root / 'npm'
        run(npm_command('install', '--global', '--prefix', npm_root, '--cache', root / 'npm-cache',
             '--ignore-scripts', '--no-audit', '--no-fund', '--offline', paths['.tgz']), root=root, env=env)
        npm_executable = [node, npm_root / 'node_modules/qiongli/bin/qiongli.mjs'] if os.name == 'nt' else npm_root / 'bin/qiongli'
        for name, executable in [('npm', npm_executable), ('pypi', python_bin / ('qiongli' + suffix))]:
            checks[name] = check_cli(executable, version=receipt['version'], root=root, env=env)
            if name == 'npm' and os.name == 'nt':
                for alias in ('qiongli', 'ql'):
                    assert (npm_root / (alias + '.cmd')).is_file()
            else:
                assert run([executable.with_name('ql' + suffix), '--version'], root=root, env=env).stdout == f"qiongli {receipt['version']}\n"
    if args.cargo_archives:
        executable = install_cargo_archives(package_root, receipt, root, env,
                                            args.cargo_target_dir or root / 'cargo-target')
        checks['cargo_archives'] = check_cli(executable, version=receipt['version'], root=root, env=env)
        checks['cargo_alias'] = check_cli(executable.with_name('ql.exe' if os.name == 'nt' else 'ql'), version=receipt['version'], root=root, env=env)
    if args.cargo_installed:
        executable = args.cargo_installed.resolve()
        checks['cargo'] = check_cli(executable, version=receipt['version'], root=root, env=env)
        checks['cargo_alias'] = check_cli(executable.with_name('ql.exe' if os.name == 'nt' else 'ql'), version=receipt['version'], root=root, env=env)
    result = {'status': 'passed', 'scope': 'local install and CLI/MCP protocol smoke; not registry publication or complete feature acceptance',
              'packages': receipt['artifacts'], 'checks': checks}
    (root / 'install-check.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
