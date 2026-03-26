---
id: beamer-builder
stage: K_presentation
description: "Generate a LaTeX Beamer presentation with theme selection, BibLaTeX citations, and math support."
inputs:
  - type: SlideDeckSpec
    description: "Slide-by-slide content specification from slide-architect"
  - type: Bibliography
    description: "BibTeX references for citations"
    required: false
  - type: FigureSpecs
    description: "Adapted slide-ready figures"
    required: false
outputs:
  - type: BeamerDeck
    artifact: "presentation/slides.tex"
  - type: BibTeXFile
    artifact: "presentation/references.bib"
constraints:
  - "Must compile with standard LaTeX distribution (TeX Live / MiKTeX)"
  - "Must use BibLaTeX for citation management"
  - "Must produce clean PDF output"
failure_modes:
  - "LaTeX distribution not installed"
  - "Missing packages requiring tlmgr install"
tools: [filesystem, code-runtime]
tags: [presentation, beamer, latex, PDF, citations, math]
domain_aware: false
---

# Beamer Builder Skill

Generate a complete LaTeX Beamer presentation from the slide specification — including theme selection, BibLaTeX citation integration, mathematical typesetting, and overlay animations.

## Related Task IDs

- `K4` (Beamer presentation generation)

## Output (contract path)

- `RESEARCH/[topic]/presentation/slides.tex`
- `RESEARCH/[topic]/presentation/references.bib`

## When to Use

- When LaTeX Beamer is the chosen backend
- When the venue expects or requires LaTeX-formatted slides
- When the presentation contains substantial mathematical notation
- When PDF is the required output format

## Process

### Step 1: Select a Beamer Theme

| Theme | Style | Best For | Preview Description |
|-------|-------|---------|-------------------|
| `metropolis` | Modern, clean, flat | CS/engineering conferences; modern look | Minimalist with progress bar |
| `Madrid` | Traditional, structured | Business/management conferences | Blue header bar with navigation |
| `Berlin` | Compact navigation | Information-dense talks | Top navigation with section dots |
| `Copenhagen` | Clean with navigation | Academic seminars | Header with section navigation |
| `CambridgeUS` | Red/black institutional | US university talks | University-style with bold colors |
| `AnnArbor` | Warm yellow/blue | Friendly presentations | Warm tones, professional |
| `default` | Minimal, no decoration | Maximum content area | Plain white, no navigation chrome |
| `Boadilla` | Compact footer | Short talks | Small footer with essential info |

> **Recommendation**: `metropolis` for modern/technical audiences, `Madrid` for traditional academic, `default` for math-heavy talks where you want maximum whitespace.

### Step 2: Document Skeleton

```latex
\documentclass[aspectratio=169]{beamer}
% aspectratio: 169 (widescreen) or 43 (traditional)

% ── Theme ──
\usetheme{metropolis}
% Optional: color customization
% \usecolortheme{owl}  % dark background
% \setbeamercolor{frametitle}{bg=blue!80!black}

% ── Packages ──
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{graphicx}
\usepackage{booktabs}          % Better tables
\usepackage{amsmath,amssymb}   % Math
\usepackage{tikz}              % Diagrams
\usepackage{appendixnumberbeamer}  % Appendix slide numbering

% ── Citations ──
\usepackage[backend=biber,style=authoryear,maxcitenames=2]{biblatex}
\addbibresource{references.bib}

% ── Metadata ──
\title[Short Title]{Full Paper Title: With Subtitle}
\subtitle{Conference / Seminar Name}
\author[Author et al.]{First Author\inst{1} \and Second Author\inst{2}}
\institute[Uni]{
  \inst{1} University A \\
  \inst{2} University B
}
\date{March 2026}

\begin{document}

% ── Title Page ──
\begin{frame}
  \titlepage
\end{frame}

% ── Outline ──
\begin{frame}{Outline}
  \tableofcontents
\end{frame}

% ── Content Frames ──
% [slides go here]

% ── References ──
\begin{frame}[allowframebreaks]{References}
  \printbibliography
\end{frame}

% ── Appendix ──
\appendix
\begin{frame}{Appendix: Robustness Checks}
  % backup slides here
\end{frame}

\end{document}
```

### Step 3: Build Frame Types

#### Section Frame
```latex
\section{Method}
% Beamer auto-generates a section title slide (theme-dependent)
```

#### Content Frame (Bullets)
```latex
\begin{frame}{Remote Work Increases Productivity by 34\%}
  \begin{itemize}
    \item Controlled for tenure, education, and department
    \item Effect is robust to multiple specifications
    \item Strongest for knowledge workers ($\beta = 0.42$)
  \end{itemize}
\end{frame}
```

#### Figure Frame
```latex
\begin{frame}{Remote Work Has a Non-Linear Effect on Productivity}
  \begin{figure}
    \centering
    \includegraphics[width=0.85\textwidth]{figures/main_result.png}
    \caption{Productivity gains by remote work intensity (95\% CI)}
  \end{figure}
\end{frame}
```

#### Table Frame
```latex
\begin{frame}{Main Regression Results}
  \begin{table}
    \centering
    \small
    \begin{tabular}{lccc}
      \toprule
      Variable & Coefficient & SE & $p$ \\
      \midrule
      Remote ratio & 0.34*** & (0.08) & $<.001$ \\
      Team size (mod.) & $-0.05$* & (0.02) & .03 \\
      Interaction & 0.12** & (0.04) & .008 \\
      \midrule
      $R^2$ & \multicolumn{3}{c}{0.23} \\
      $N$ & \multicolumn{3}{c}{5,000} \\
      \bottomrule
    \end{tabular}
  \end{table}
\end{frame}
```

#### Two-Column Frame
```latex
\begin{frame}{Comparing Methods}
  \begin{columns}[T]
    \column{0.48\textwidth}
    \textbf{Quantitative}
    \begin{itemize}
      \item $N = 5{,}000$ employees
      \item Panel data (2019--2023)
      \item Fixed effects regression
    \end{itemize}

    \column{0.48\textwidth}
    \textbf{Qualitative}
    \begin{itemize}
      \item 28 semi-structured interviews
      \item Thematic analysis
      \item Member checking
    \end{itemize}
  \end{columns}
\end{frame}
```

#### Block / Theorem Frame
```latex
\begin{frame}{Theoretical Prediction}
  \begin{theorem}[Proposition 1]
    If autonomy increases with remote work, and autonomy mediates
    the work-arrangement--productivity relationship, then remote work
    will increase productivity.
  \end{theorem}
  
  \begin{alertblock}{Key Implication}
    The effect should be strongest for knowledge workers
    with high task autonomy.
  \end{alertblock}
\end{frame}
```

#### Quote Frame
```latex
\begin{frame}{}
  \begin{center}
    \Large\itshape
    ``The future of work is not about where you work, \\
    but how you work.''
    
    \vspace{1em}
    \normalsize\upshape
    --- \textcite{smith2020}
  \end{center}
\end{frame}
```

### Step 4: Use Overlays and Animations

#### Pause (Sequential Reveal)
```latex
\begin{frame}{Three Key Contributions}
  \begin{enumerate}
    \item \textbf{Theoretical}: Extends autonomy theory \pause
    \item \textbf{Methodological}: Novel WFH productivity instrument \pause
    \item \textbf{Practical}: Evidence-based hybrid work policy
  \end{enumerate}
\end{frame}
```

#### onslide (Selective Visibility)
```latex
\begin{frame}{Analysis Pipeline}
  \begin{enumerate}
    \item<1-> Clean raw data
    \item<2-> Merge panel waves
    \item<3-> Estimate DiD model
    \item<4-> Run robustness checks
  \end{enumerate}
\end{frame}
```

#### only (Replacement)
```latex
\begin{frame}{Results by Group}
  \only<1>{
    \includegraphics[width=\textwidth]{figures/result_all.png}
  }
  \only<2>{
    \includegraphics[width=\textwidth]{figures/result_knowledge.png}
  }
  \only<3>{
    \includegraphics[width=\textwidth]{figures/result_routine.png}
  }
\end{frame}
```

#### alert (Highlighting)
```latex
\begin{frame}{Key Variables}
  \begin{itemize}
    \item Outcome: Productivity index
    \item Treatment: \alert<2>{Remote work ratio}
    \item Moderator: Team size
  \end{itemize}
\end{frame}
```

### Step 5: Citations with BibLaTeX

```latex
% In-text (narrative): "Smith (2020) found..."
\textcite{smith2020} found that remote work...

% Parenthetical: "...increases productivity (Smith, 2020)"
...increases productivity \parencite{smith2020}.

% Multiple: "(Smith, 2020; Jones, 2021)"
\parencites{smith2020}{jones2021}

% Page-specific: "(Smith, 2020, p. 15)"
\parencite[p.~15]{smith2020}
```

### Step 6: Math Support

```latex
% Inline math
The estimating equation is $Y_{it} = \alpha + \beta D_{it} + \gamma X_{it} + \epsilon_{it}$.

% Display math
\begin{frame}{Identification Strategy}
  \begin{equation}
    Y_{it} = \alpha_i + \lambda_t + \beta \cdot \text{Remote}_{it}
    + \mathbf{X}_{it}'\boldsymbol{\gamma} + \varepsilon_{it}
  \end{equation}
  where $\alpha_i$ are individual fixed effects and $\lambda_t$ are time fixed effects.
\end{frame}

% Aligned equations
\begin{frame}{Model Specifications}
  \begin{align}
    \text{Model 1:} \quad Y_{it} &= \alpha + \beta D_{it} + \varepsilon_{it} \\
    \text{Model 2:} \quad Y_{it} &= \alpha_i + \beta D_{it} + \mathbf{X}'\gamma + \varepsilon_{it} \\
    \text{Model 3:} \quad Y_{it} &= \alpha_i + \lambda_t + \beta D_{it} + \mathbf{X}'\gamma + \varepsilon_{it}
  \end{align}
\end{frame}
```

### Step 7: Color Customization

```latex
% Custom colors
\definecolor{uniblue}{RGB}{0, 51, 102}
\definecolor{unigold}{RGB}{204, 153, 0}

% Apply to beamer elements
\setbeamercolor{frametitle}{bg=uniblue, fg=white}
\setbeamercolor{title}{fg=uniblue}
\setbeamercolor{structure}{fg=uniblue}
\setbeamercolor{alerted text}{fg=unigold}
```

### Step 8: Compile

```bash
# Standard compilation (with BibLaTeX)
pdflatex slides.tex
biber slides
pdflatex slides.tex
pdflatex slides.tex

# Or use latexmk for automated compilation
latexmk -pdf slides.tex

# With XeLaTeX (for custom fonts)
xelatex slides.tex
biber slides
xelatex slides.tex
```

### Step 9: Speaker Notes (Optional)

```latex
% Enable notes
\usepackage{pgfpages}
\setbeameroption{show notes on second screen=right}

% Add notes to individual frames
\begin{frame}{Title}
  Content...
\end{frame}
\note{
  Key points to mention:
  - Start with the hook about productivity
  - Mention the 34\% figure
  - Transition: "But this raises the question..."
}
```

## Quality Bar

The Beamer deck is **ready** when:

- [ ] Compiles without errors: `pdflatex` + `biber` + `pdflatex` × 2
- [ ] Theme selected and consistently applied
- [ ] All citations resolve via BibLaTeX + `references.bib`
- [ ] Figures included with correct paths and sizing
- [ ] Tables use `booktabs` formatting
- [ ] Math renders correctly (no package conflicts)
- [ ] Overlays/animations used purposefully
- [ ] Appendix slides use `\appendix` command (correct numbering)
- [ ] PDF output reviewed for formatting issues
- [ ] Total slides match the presentation plan time budget

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Missing `biber` run | Citations show as `[?]` | Run full compilation: `pdflatex → biber → pdflatex → pdflatex` |
| Wrong aspect ratio | Slides don't match projector | Use `aspectratio=169` for widescreen, `43` for 4:3 |
| Overfull hbox warnings | Text or images exceed frame width | Reduce `\includegraphics` width or use `\small` |
| Package conflicts | Math packages clash with theme packages | Load `amsmath` before theme-specific packages |
| Appendix slides counted in total | "Slide 15 of 25" when only 15 are presented | Use `\appendix` + `appendixnumberbeamer` package |
| Hardcoded paths | Compilation fails on different machine | Use relative paths for all figures |
| Too many overlays | Presentation feels sluggish | Only animate for comprehension; max 3–4 clicks per frame |
