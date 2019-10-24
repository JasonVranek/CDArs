use crate::order::{Order, OrderType, TradeType};
use crate::exchange::queue::Queue;
use crate::exchange::order_book::Book;
use crate::controller::{Task, State};
use crate::exchange::auction::{Auction};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Mutex, Arc};

pub struct QueueProcessor {}

impl QueueProcessor {
	// Concurrently process orders in the queue. Each order is
	// either of OrderType::{Enter, Update, Cancel}. Each order will
	// modify the state of either the Bids or Asks Book, but must
	// first acquire a lock on the respective book. 
	pub fn conc_process_order_queue(queue: Arc<Queue>, 
									bids: Arc<Book>, 
									asks: Arc<Book>) 
									-> Vec<JoinHandle<()>>{
		// Acquire lock of Queue
		// Pop off contents of Queue
		// match over the OrderType
		// process each order based on OrderType
		let mut handles = Vec::<JoinHandle<()>>::new();
		for order in queue.pop_all() {
			let handle = match order.order_type {
				OrderType::Enter => QueueProcessor::process_enter(Arc::clone(&bids), Arc::clone(&asks), order),
				OrderType::Update => QueueProcessor::process_update(Arc::clone(&bids), Arc::clone(&asks), order),
				OrderType::Cancel => QueueProcessor::process_cancel(Arc::clone(&bids), Arc::clone(&asks), order),
			};
			handles.push(handle);
		}
		handles
	}


	// Checks if the new order crosses. Modifies orders in book then calculates new max price
	fn process_enter(bids: Arc<Book>, asks: Arc<Book>, order: Order) -> JoinHandle<()> {
		// Spawn a new thread to process the order
	    thread::spawn(move || {
			// Since CDA we will check if the order transacts here:
			match order.trade_type {
				TradeType::Ask => {
					// Only check for cross if this ask price is lower than best ask
					if order.price < asks.get_min_price() {
						// This will add the new ask to the book if it doesn't fully transact
						Auction::calc_ask_crossing(bids, asks, order);
					} else {
						// We need to add the ask to the book, best price will be updated in add_order
						asks.add_order(order).expect("Failed to add order");
					}
				},
				TradeType::Bid => {
					// Only check for cross if this bid price is higher than best bid
					if order.price > bids.get_max_price() {
						// This will add the new bid to the book if it doesn't fully transact
						Auction::calc_bid_crossing(bids, asks, order);
					} else {
						// We need to add the ask to the book, best price will be updated in add_order
						bids.add_order(order).expect("Failed to add order...");
					}
				}
			}
	    })
	}

	// Cancels the previous order and then enters this as a new one
	// Updates an order in the Bids or Asks Book in it's own thread
	fn process_update(bids: Arc<Book>, asks: Arc<Book>, order: Order) -> JoinHandle<()> {
		// update books min/max price if this overwrites current min/max OR this order contains new min/max
	    thread::spawn(move || {
			match order.trade_type {
				TradeType::Ask => {
					// Cancel the orginal order:
					match asks.cancel_order_by_index(&order.trader_id) {
						Ok(()) => {},
						Err(e) => println!("{:?}", e),
					}
					// Only check for cross if this ask price is lower than best ask
					if order.price < asks.get_min_price() {
						// This will add the new ask to the book if it doesn't fully transact
						Auction::calc_ask_crossing(bids, asks, order);
					} else {
						// We need to add the ask to the book, best price will be updated in add_order
						asks.add_order(order).expect("Failed to add order");
					}
				},
				TradeType::Bid => {
					// Cancel the orginal order:
					match asks.cancel_order_by_index(&order.trader_id) {
						Ok(()) => {},
						Err(e) => println!("{:?}", e),
					}
					// Only check for cross if this bid price is higher than best bid
					if order.price > bids.get_max_price() {
						// This will add the new bid to the book if it doesn't fully transact
						Auction::calc_bid_crossing(bids, asks, order);
					} else {
						// We need to add the ask to the book, best price will be updated in add_order
						bids.add_order(order).expect("Failed to add order...");
					}
				}
			}
	    })
	}

	// Cancels the order living in the Bids or Asks Book
	fn process_cancel(bids: Arc<Book>, asks: Arc<Book>, order: Order) -> JoinHandle<()> {
	    thread::spawn(move || {
			let book = match order.trade_type {
				TradeType::Ask => asks,
				TradeType::Bid => bids,
			};
			
			// If the cancel fails bubble error up.
			match book.cancel_order(order) {
	    		Ok(()) => {},
	    		Err(e) => {
	    			println!("ERROR: {}", e);
	    			// TODO send an error response over TCP
	    		}
	    	}
	    })
	}

	pub fn async_queue_task(queue: Arc<Queue>, 
							bids: Arc<Book>, 
							asks: Arc<Book>, 
							state: Arc<Mutex<State>>, 
							duration: u64) -> Task
	{
	    Task::rpt_task(move || {
	    	match *state.lock().expect("Couldn't lock state in queue task") {
				State::Process => {
					let handles = QueueProcessor::conc_process_order_queue(Arc::clone(&queue), 
								Arc::clone(&bids),
								Arc::clone(&asks));

					for h in handles {
						h.join().expect("Couldn't join queue tasks");
					}
					// println!("Processing order queue");
				},
				State::Auction => println!("Can't process order queue because auction!"),
				State::PreAuction => println!("Can't process order queue because pre-auction!"),
			}
	    }, duration)
	}
}
