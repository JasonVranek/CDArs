// extern crate <name_of_my_crate_to_test>
use flow_rs::exchange::queue_processing::QueueProcessor;
use flow_rs::exchange::order_processing::OrderProcessor;
use flow_rs::exchange::order_processing::*;
use flow_rs::order::*;
use flow_rs::exchange::auction::Auction;
use std::sync::Arc;

// Include the common module for setting up state for tests
mod common;



#[test]
fn default_test() {
	common::setup();
	assert_eq!(1, 1);
}

#[test]
fn test_add_order_to_book() {
	let bid = common::setup_bid_order();

	let book = common::setup_bids_book();

	book.add_order(bid);

	assert_eq!(book.len(), 1);

	let order = book.orders.lock().unwrap().pop().unwrap();

}


#[test]
fn test_conc_queue_recv_order() {
	// Setup a queue
	let queue = Arc::new(common::setup_queue());

	let mut order = common::setup_bid_order();

	// Mutate order
	order.price = 199.0;

	// Accept order in a new thread
	let handle = OrderProcessor::conc_recv_order(order, Arc::clone(&queue));

	// Wait for thread to finish
	handle.join().unwrap();

	// Confirm the queue's order is correct
	let order = queue.pop().unwrap();

	assert_eq!(order.price, 199.0);
}

#[test]
fn test_queue_pop_all() {
	let queue = common::setup_full_queue();
	let popped_off = queue.pop_all();
	assert_eq!(popped_off.len(), 3);
}

#[test]
fn test_ask_transaction() {
	// Setup queue and order books
	let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let num_bids: usize = 100;
	let (bids, asks) = common::setup_ask_cross_orders(num_bids);
	let mut handles = Vec::new();

	// Send all the bid orders in parallel 
	for bid in bids {
		handles.push(OrderProcessor::conc_recv_order(bid, Arc::clone(&queue)));
	}

	// Wait for the threads to finish
	for h in handles {
		h.join().unwrap();
	}

	// Process all of the bid orders in the queue
	let mut handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));

	for h in handles {
		h.join().unwrap();
	}						

	// There should be num_bids bids in the book, with max price num_bids and quantity 5.0
	assert_eq!(bids_book.len(), num_bids);
	let mut b_max_price = bids_book.get_max_price();
	assert_eq!(b_max_price, num_bids as f64);

	let mut handles = Vec::new();
	// Send two asks orders
	for ask in asks {
		handles.push(OrderProcessor::conc_recv_order(ask, Arc::clone(&queue)));
	}

	for h in handles {
		h.join().unwrap();
	}

	// Process the new ask orders
	let mut handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));
	
	for h in handles {
		h.join().unwrap();
	}

	// Only one ask should cross and fill, other will remain
	assert_eq!(asks_book.len(), 1);

	// The filled ask had 10x quantity as the bids so should have filled 10 bids
	assert_eq!(bids_book.len(), num_bids - 10);
	b_max_price = bids_book.get_max_price();

	let a_min_price = asks_book.get_min_price();
	assert_eq!(b_max_price, num_bids as f64 - 10.0);

	// Min price set by remaining ask
	assert_eq!(a_min_price, num_bids as f64 * 1000.0)
}


#[test]
fn test_bid_transaction() {
	// Setup queue and order books
	let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let num_asks: usize = 100;
	let (bids, asks) = common::setup_bid_cross_orders(num_asks);
	let mut handles = Vec::new();

	// Send all the ask orders in parallel 
	for ask in asks {
		handles.push(OrderProcessor::conc_recv_order(ask, Arc::clone(&queue)));
	}

	// Wait for the threads to finish
	for h in handles {
		h.join().unwrap();
	}

	// Process all of the bid orders in the queue
	let mut handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));

	for h in handles {
		h.join().unwrap();
	}						

	// There should be num_asks asks in the book, with min price 50 -> 50 + num_asks and quantity 5.0
	assert_eq!(asks_book.len(), num_asks);
	let mut a_min_price = asks_book.get_min_price();
	assert_eq!(a_min_price, 51.0);

	let mut handles = Vec::new();
	// Send two bid orders
	for bid in bids {
		handles.push(OrderProcessor::conc_recv_order(bid, Arc::clone(&queue)));
	}

	for h in handles {
		h.join().unwrap();
	}

	// Process the new ask orders
	let mut handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));
	
	for h in handles {
		h.join().unwrap();
	}

	// Only one bid should cross and fill, other will remain
	assert_eq!(bids_book.len(), 1);

	// The filled bid had 10x quantity as the asks so should have filled 10 asks
	assert_eq!(asks_book.len(), num_asks - 10);
	a_min_price = asks_book.get_min_price();
	assert_eq!(a_min_price, 61.0);

	// Max price set by remaining bid
	let b_max_price = bids_book.get_max_price();
	assert_eq!(b_max_price, 0.0)
}












#[test]
pub fn test_update_order() {
    let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let (mut bids, asks) = common::setup_orders();
	bids[0].trader_id = format!("jason");
	let mut handles = Vec::new();

	// Send all the orders in parallel 
	for bid in bids {
		handles.push(OrderProcessor::conc_recv_order(bid, Arc::clone(&queue)));
	}
	for ask in asks {
		handles.push(OrderProcessor::conc_recv_order(ask, Arc::clone(&queue)));
	}

	// Wait for the threads to finish
	for h in handles {
		h.join().unwrap();
	}

	// Process all of the orders in the queue
	let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));

	for h in handles {
		h.join().unwrap();
	}

	assert_eq!(bids_book.len(), 100);
	assert_eq!(asks_book.len(), 100);

	// Create a new order to update book 
	let mut update_order = common::setup_bid_order();
	update_order.trader_id = format!("jason");
	update_order.order_type = OrderType::Update;
	update_order.price = 99.9;
	update_order.quantity = 555.5;

	// Send new order to queue
	OrderProcessor::conc_recv_order(update_order, Arc::clone(&queue)).join().unwrap();

	// Process queue
	let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));
	for h in handles {
		h.join().unwrap();
	}

	// Books should be same length
	assert_eq!(bids_book.len(), 100);
	assert_eq!(asks_book.len(), 100);

	// Find the order with id "jason"
	let index = bids_book.peek_id_pos(format!("jason"));

	// Unwrap the index and check order has been updating
	if let Some(i) = index {
		let order = &bids_book.orders.lock().unwrap()[i];
		assert_eq!(order.trader_id, format!("jason"));
		assert_eq!(order.price, 99.9);
		assert_eq!(order.quantity, 555.5);
		assert_eq!(order.order_type, OrderType::Update);
	} else {
		panic!("Update Order should exist");
	}

}

#[test]
pub fn test_cancel_order() {
    let queue = Arc::new(common::setup_queue());
	let bids_book = Arc::new(common::setup_bids_book());
	let asks_book = Arc::new(common::setup_asks_book());
	
	// Setup bids and asks
	let (mut bids, asks) = common::setup_orders();
	bids[0].trader_id = format!("jason");
	bids[0].price = 99999.9;
	bids[0].quantity = -1.0; // negative to test a negative quantity
	let mut handles = Vec::new();

	// Send all the orders in parallel 
	for bid in bids {
		handles.push(OrderProcessor::conc_recv_order(bid, Arc::clone(&queue)));
	}
	for ask in asks {
		handles.push(OrderProcessor::conc_recv_order(ask, Arc::clone(&queue)));
	}

	// Wait for the threads to finish
	for h in handles {
		h.join().unwrap();
	}

	// Process all of the orders in the queue
	let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));

	for h in handles {
		h.join().unwrap();
	}

	assert_eq!(bids_book.len(), 100);
	assert_eq!(asks_book.len(), 100);

	// New max price will be equal to mutated order 
	assert_eq!(bids_book.get_max_price(), 99999.9);
	// assert_eq!(bids_book.get_min_price(), -1.0);

	// Create a new order to update book 
	let mut update_order = common::setup_bid_order();
	update_order.trader_id = format!("jason");
	update_order.price = 99999.9;
	update_order.order_type = OrderType::Cancel;
	update_order.quantity = -1.0; // negative to test a low min price

	// Send new order to queue
	OrderProcessor::conc_recv_order(update_order, Arc::clone(&queue)).join().unwrap();

	// Process queue
	let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
							Arc::clone(&bids_book),
							Arc::clone(&asks_book));
	for h in handles {
		h.join().unwrap();
	}

	// Books should be same length
	assert_eq!(bids_book.len(), 99);
	assert_eq!(asks_book.len(), 100);

	// Find the order with id "jason"
	let index = bids_book.peek_id_pos(format!("jason"));

	// Unwrap the index and check order has been updating
	if let Some(_) = index {
		panic!("Cancel Order should not exist anymore");
	} 

	// The new max price will be updated to something lower once order has been cancelled
	assert_ne!(bids_book.get_max_price(), 99999.9);

	// New min price will be 1.0 since orders iterated from p_lows 0..100 and we mutated the 0th order
	assert_ne!(bids_book.get_min_price(), -1.0);
	assert_eq!(bids_book.get_min_price(), 1.0);
}




















