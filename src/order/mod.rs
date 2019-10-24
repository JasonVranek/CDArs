/// Enum for matching over order types
#[derive(Debug, PartialEq)]
pub enum OrderType {
    Enter,
    Update,
    Cancel,
}

impl Clone for OrderType {
	fn clone(&self) -> OrderType { 
		match self {
			OrderType::Enter => OrderType::Enter,
			OrderType::Update => OrderType::Update,
			OrderType::Cancel => OrderType::Cancel,
		}
	}
}


// Enum for matching over bid or ask
#[derive(Debug, PartialEq)]
pub enum TradeType {
    Bid,
    Ask,
}

impl Clone for TradeType {
	fn clone(&self) -> TradeType { 
		match self {
			TradeType::Ask => TradeType::Ask,
			TradeType::Bid => TradeType::Bid,
		}
	}
}

/// The internal data structure that the CDA market operates on. 
/// trader_id: String -> identifier of the trader and their order
/// order_type: OrderType{Enter, Update, Cancel} -> identifies how the order is used by the exchange
/// trade_type: TradeType{Bid, Ask} -> decides which order book the order is placed in 
/// price: f64 -> trader's willing ness to buy or sell
/// quantity: f64 -> amount of shares to buy/sell
pub struct Order {
	pub trader_id: String,		
	pub order_type: OrderType,	
	pub trade_type: TradeType,  
	pub price: f64,				
	pub quantity: f64,			
}

impl Order {
    pub fn new(t_id: String, o_t: OrderType, t_t: TradeType, p: f64, q: f64) -> Order
    {
    	Order {
    		trader_id: t_id,		
			order_type: o_t,	
			trade_type: t_t,  
			price: p,				
			quantity: q,	
    	}
    }

    pub fn describe(&self) {
    	println!("Trader Id: {:?} \n OrderType: {:?}
    		price: {:?}, quantity: {:?}", 
    		self.trader_id, self.order_type,
    		self.price, self.quantity);
    }
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_order() {
		let order = Order::new(
			String::from("trader_id"),
			OrderType::Enter,
			TradeType::Bid,
			50.0,
			500.0,
		);

		assert_eq!(order.trader_id, "trader_id");
		assert_eq!(order.order_type, OrderType::Enter);
		assert_eq!(order.trade_type, TradeType::Bid);
		assert_eq!(order.price, 50.0);
		assert_eq!(order.quantity, 500.0);
	}
}

























