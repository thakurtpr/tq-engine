//! Broker interface. Only `Paper` exists; a live broker needs explicit human sign-off.
//! `submit` takes `ApprovedOrder`, so nothing opens exposure without passing the risk gate.
//! `close` is the one ungated path, and it is reduce-only: it can never open or grow a position.

use std::collections::HashMap;

use rust_decimal::Decimal;
use thiserror::Error;
use tq_core::{Fill, Price, Qty, Side, Symbol, Timestamp};
use tq_costs::CostSchedule;
use tq_risk::ApprovedOrder;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BrokerError {
    #[error("order rejected by broker: {0}")]
    Rejected(String),
}

pub trait Broker {
    fn submit(&mut self, order: ApprovedOrder, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError>;

    /// Sells `qty` of an existing long. Not risk-gated, because exits must always be possible,
    /// so implementations must reject anything that would open or increase exposure.
    fn close(&mut self, symbol: &Symbol, qty: Qty, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError>;
}

/// Simulated broker. Fills at the price the caller supplies and charges `CostSchedule` costs.
#[derive(Debug)]
pub struct Paper {
    costs: CostSchedule,
    fills: Vec<Fill>,
    holdings: HashMap<Symbol, Qty>,
}

impl Default for Paper {
    /// Zerodha NSE delivery costs: the conservative default for swing backtests.
    fn default() -> Self {
        Self::new(CostSchedule::zerodha_nse_delivery())
    }
}

impl Paper {
    pub fn new(costs: CostSchedule) -> Self {
        Self { costs, fills: Vec::new(), holdings: HashMap::new() }
    }

    pub fn fills(&self) -> &[Fill] {
        &self.fills
    }

    /// Net quantity held (negative if short via an approved sell order).
    pub fn holding(&self, symbol: &Symbol) -> Qty {
        self.holdings.get(symbol).copied().unwrap_or(Decimal::ZERO)
    }

    fn record(&mut self, symbol: &Symbol, side: Side, qty: Qty, ts: Timestamp, price: Price) -> Fill {
        let fill = Fill {
            ts,
            symbol: symbol.clone(),
            side,
            qty,
            price,
            costs: self.costs.compute(side, price * qty).total(),
        };
        let signed = match side {
            Side::Buy => qty,
            Side::Sell => -qty,
        };
        let held = self.holding(symbol) + signed;
        self.holdings.insert(symbol.clone(), held);
        self.fills.push(fill.clone());
        fill
    }
}

impl Broker for Paper {
    fn submit(&mut self, order: ApprovedOrder, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError> {
        let o = order.order();
        Ok(self.record(&o.symbol, o.side, o.qty, ts, fill_price))
    }

    fn close(&mut self, symbol: &Symbol, qty: Qty, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError> {
        if qty <= Decimal::ZERO {
            return Err(BrokerError::Rejected("close qty must be positive".into()));
        }
        if qty > self.holding(symbol) {
            return Err(BrokerError::Rejected(format!("close of {qty} exceeds long holding in {}", symbol.0)));
        }
        Ok(self.record(symbol, Side::Sell, qty, ts, fill_price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::{OrderRequest, Side, Symbol};
    use tq_risk::{AccountState, RiskGate, RiskLimits};

    fn sbin() -> Symbol {
        Symbol("NSE:SBIN".into())
    }

    fn approved_buy(qty: Qty) -> ApprovedOrder {
        let order = OrderRequest::new(sbin(), Side::Buy, qty, dec!(800), dec!(780)).unwrap();
        let acct = AccountState::fresh(dec!(100000)).unwrap();
        RiskGate::new(RiskLimits::default()).check(order, &acct).unwrap()
    }

    #[test]
    fn paper_records_fill_for_approved_order() {
        let mut paper = Paper::default();
        let fill = paper.submit(approved_buy(dec!(5)), 1, dec!(801)).unwrap();

        assert_eq!(fill.price, dec!(801));
        assert_eq!(paper.fills().len(), 1);
    }

    #[test]
    fn paper_charges_cost_schedule_on_fill_turnover() {
        let schedule = CostSchedule::zerodha_nse_delivery();
        let mut paper = Paper::new(schedule.clone());
        let fill = paper.submit(approved_buy(dec!(10)), 1, dec!(810)).unwrap();

        let expected = schedule.compute(Side::Buy, dec!(8100)).total();
        assert!(expected > dec!(0));
        assert_eq!(fill.costs, expected);
    }

    #[test]
    fn close_sells_held_qty_with_sell_side_costs() {
        let mut paper = Paper::default();
        paper.submit(approved_buy(dec!(10)), 1, dec!(800)).unwrap();
        let fill = paper.close(&sbin(), dec!(10), 2, dec!(820)).unwrap();

        let expected = CostSchedule::zerodha_nse_delivery().compute(Side::Sell, dec!(8200)).total();
        assert_eq!(fill.side, Side::Sell);
        assert_eq!(fill.costs, expected);
        assert_eq!(paper.holding(&sbin()), dec!(0));
    }

    #[test]
    fn close_is_reduce_only_and_cannot_open_a_short() {
        let mut paper = Paper::default();
        assert!(paper.close(&sbin(), dec!(1), 1, dec!(800)).is_err());
        paper.submit(approved_buy(dec!(5)), 1, dec!(800)).unwrap();
        assert!(paper.close(&sbin(), dec!(6), 2, dec!(800)).is_err());
        assert_eq!(paper.holding(&sbin()), dec!(5));
    }
}
