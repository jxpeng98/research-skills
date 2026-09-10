"""
Tier 1 Standard Template: Portfolio Optimization (Finance)

Uses classical Mean-Variance Optimization.
Method: CAPM / Efficient Frontier
Library: PyPortfolioOpt
"""

import pandas as pd
import numpy as np
from pypfopt import EfficientFrontier
from pypfopt import risk_models
from pypfopt import expected_returns
from pypfopt.discrete_allocation import DiscreteAllocation, get_latest_prices

def optimize_portfolio(price_data: pd.DataFrame, risk_aversion: float = 1.0):
    """
    Perform Mean-Variance Optimization.
    
    Args:
        price_data: DataFrame with date index and stock tickers as columns
        risk_aversion: Parameter for risk aversion (higher = safer)
    """
    # 1. Calculate Expected Returns and Sample Covariance
    mu = expected_returns.mean_historical_return(price_data)
    S = risk_models.sample_cov(price_data)

    # 2. Optimize for Maximal Sharpe Ratio
    ef = EfficientFrontier(mu, S)
    
    # Option: Custom objective (e.g. min volatility)
    # weights = ef.min_volatility()
    
    weights = ef.max_sharpe()
    cleaned_weights = ef.clean_weights()

    print("Optimized Weights:")
    print(cleaned_weights)
    
    ef.portfolio_performance(verbose=True)
    return cleaned_weights

if __name__ == "__main__":
    # Example usage
    # prices = pd.read_csv("stock_prices.csv", parse_dates=True, index_col="date")
    # optimize_portfolio(prices)
    pass
