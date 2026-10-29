//! Broker interface. Only `Paper` exists; a live broker needs explicit human sign-off.
//! `submit` takes `ApprovedOrder`, so nothing reaches a broker without passing the risk gate.

use thiserror::Error;
use tq_core::{Fill, Price, Timestamp};
use tq_risk::ApprovedOrder;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BrokerError {
    #[error("order rejected by broker: {0}")]
    Rejected(String),
}

pub trait Broker {
    fn submit(&mut self, order: ApprovedOrder, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError>;
}

/// Simulated broker. Fills at the price the caller supplies; costs come from the cost model later.
#[derive(Debug, Default)]
pub struct Paper {
    fills: Vec<Fill>,
}

impl Paper {
    pub fn fills(&self) -> &[Fill] {
        &self.fills
    }
}

impl Broker for Paper {
    fn submit(&mut self, order: ApprovedOrder, ts: Timestamp, fill_price: Price) -> Result<Fill, BrokerError> {
        let o = order.order();
        let fill = Fill {
            ts,
            symbol: o.symbol.clone(),
            side: o.side,
            qty: o.qty,
            price: fill_price,
            costs: Default::default(),
        };
        self.fills.push(fill.clone());
        Ok(fill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::{OrderRequest, Side, Symbol};
    use tq_risk::{AccountState, RiskGate, RiskLimits};

    #[test]
    fn paper_records_fill_for_approved_order() {
        let acct = AccountState {
            equity: dec!(100000),
            start_of_day_equity: dec!(100000),
            peak_equity: dec!(100000),
            open_positions: 0,
            kill_switch: false,
        };
        let order = OrderRequest::new(Symbol("NSE:SBIN".into()), Side::Buy, dec!(5), dec!(800), dec!(780)).unwrap();
        let approved = RiskGate::new(RiskLimits::default()).check(order, &acct).unwrap();

        let mut paper = Paper::default();
        let fill = paper.submit(approved, 1, dec!(801)).unwrap();

        assert_eq!(fill.price, dec!(801));
        assert_eq!(paper.fills().len(), 1);
    }
}
