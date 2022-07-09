use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,

    #[serde(rename = "withdrawal")]
    Withdrawal,

    #[serde(rename = "dispute")]
    Dispute,

    #[serde(rename = "resolve")]
    Resolve,

    #[serde(rename = "chargeback")]
    Chargeback,
}
pub type Client = u16;
pub type TransactionId = u32;
pub type Amount = Decimal;
pub type TransactionInfo = (Amount, bool);

pub struct Account {
    pub(crate) available: Amount,
    pub(crate) held: Amount,
    pub(crate) total: Amount,
    pub(crate) locked: bool,
}

#[derive(Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: Client,
    tx: TransactionId,
    amount: Option<Amount>,
}
impl Transaction {
    pub fn tx_type(&self) -> TransactionType {
        self.tx_type.clone()
    }

    pub fn client(&self) -> Client {
        self.client
    }

    pub fn tx(&self) -> TransactionId {
        self.tx
    }

    pub fn amount(&self) -> Option<Decimal> {
        self.amount
    }
}