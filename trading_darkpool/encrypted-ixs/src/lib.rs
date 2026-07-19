use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    const ORDER_BOOK_SIZE: usize = 10;

    pub struct Order {
        pub bid:    u64,
        pub size:   u64,
        pub is_buy: bool,
        pub owner:  [u8; 32],
        pub active: bool,
    }

    pub struct OrderBook {
        pub orders: [Order; ORDER_BOOK_SIZE],
        pub count:  u64,
    }

    pub struct NewOrder {
        pub bid:    u64,
        pub size:   u64,
        pub is_buy: bool,
        pub owner:  [u8; 32],
    }

    pub struct MatchResult {
        pub buy_owner:     [u8; 32],
        pub sell_owner:    [u8; 32],
        pub matched_size:  u64,
        pub matched_price: u64,
        pub matched:       bool,
    }

    pub struct BookStats {
        pub total_orders: u64,
        pub buy_volume:   u64,
        pub sell_volume:  u64,
    }

    #[instruction]
    pub fn place_order(
        book_ctxt:  Enc<Shared, OrderBook>,
        order_ctxt: Enc<Shared, NewOrder>,
    ) -> Enc<Shared, OrderBook> {
        let mut book    = book_ctxt.to_arcis();
        let new_order   = order_ctxt.to_arcis();
        let mut placed  = false;

        for i in 0..ORDER_BOOK_SIZE {
            if !book.orders[i].active && !placed {
                book.orders[i].bid    = new_order.bid;
                book.orders[i].size   = new_order.size;
                book.orders[i].is_buy = new_order.is_buy;
                book.orders[i].owner  = new_order.owner;
                book.orders[i].active = true;
                book.count += 1;
                placed = true;
            }
        }

        book_ctxt.owner.from_arcis(book)
    }

    #[instruction]
    pub fn match_orders(
        book_ctxt: Enc<Shared, OrderBook>,
    ) -> Enc<Shared, MatchResult> {
        let book = book_ctxt.to_arcis();

        let mut result = MatchResult {
            buy_owner:     [0u8; 32],
            sell_owner:    [0u8; 32],
            matched_size:  0,
            matched_price: 0,
            matched:       false,
        };

        for i in 0..ORDER_BOOK_SIZE {
            if book.orders[i].active && book.orders[i].is_buy && !result.matched {
                for j in 0..ORDER_BOOK_SIZE {
                    if book.orders[j].active
                        && !book.orders[j].is_buy
                        && !result.matched
                        && book.orders[i].bid >= book.orders[j].bid
                    {
                        let size = if book.orders[i].size < book.orders[j].size {
                            book.orders[i].size
                        } else {
                            book.orders[j].size
                        };
                        result.buy_owner     = book.orders[i].owner;
                        result.sell_owner    = book.orders[j].owner;
                        result.matched_size  = size;
                        result.matched_price = (book.orders[i].bid + book.orders[j].bid) / 2;
                        result.matched       = true;
                    }
                }
            }
        }

        book_ctxt.owner.from_arcis(result)
    }

    #[instruction]
    pub fn cancel_order(
        book_ctxt:  Enc<Shared, OrderBook>,
        owner_ctxt: Enc<Shared, [u8; 32]>,
    ) -> Enc<Shared, OrderBook> {
        let mut book      = book_ctxt.to_arcis();
        let     owner     = owner_ctxt.to_arcis();
        let mut cancelled = false;

        for i in 0..ORDER_BOOK_SIZE {
            if book.orders[i].active
                && book.orders[i].owner == owner
                && !cancelled
            {
                book.orders[i].active = false;
                book.orders[i].bid    = 0;
                book.orders[i].size   = 0;
                if book.count > 0 {
                    book.count -= 1;
                }
                cancelled = true;
            }
        }

        book_ctxt.owner.from_arcis(book)
    }

    #[instruction]
    pub fn get_stats(
        book_ctxt: Enc<Shared, OrderBook>,
    ) -> Enc<Shared, BookStats> {
        let book = book_ctxt.to_arcis();

        let mut stats = BookStats {
            total_orders: 0,
            buy_volume:   0,
            sell_volume:  0,
        };

        for i in 0..ORDER_BOOK_SIZE {
            if book.orders[i].active {
                stats.total_orders += 1;
                if book.orders[i].is_buy {
                    stats.buy_volume  += book.orders[i].size;
                } else {
                    stats.sell_volume += book.orders[i].size;
                }
            }
        }

        book_ctxt.owner.from_arcis(stats)
    }

    pub struct ReputationData {
        pub completed_trades: u64,
        pub disputes_lost:    u64,
        pub score:            u64,
    }

    pub struct ReputationEvent {
        pub is_completion: bool,
    }

    #[instruction]
    pub fn update_reputation(
        data_ctxt:  Enc<Shared, ReputationData>,
        event_ctxt: Enc<Shared, ReputationEvent>,
    ) -> Enc<Shared, ReputationData> {
        let mut data = data_ctxt.to_arcis();
        let event    = event_ctxt.to_arcis();

        if event.is_completion {
            data.completed_trades += 1;
        } else {
            data.disputes_lost += 1;
        }

        let completion_score = data.completed_trades * 10;
        let dispute_penalty  = data.disputes_lost * 25;
        let raw_score = if completion_score > dispute_penalty {
            completion_score - dispute_penalty
        } else {
            0
        };
        data.score = if raw_score > 1000 { 1000 } else { raw_score };

        data_ctxt.owner.from_arcis(data)
    }

    pub struct ThresholdCheck {
        pub score:     u64,
        pub min_score: u64,
    }

    #[instruction]
    pub fn check_threshold(request: Enc<Shared, ThresholdCheck>) -> Enc<Shared, u64> {
        let r = request.to_arcis();
        let passes = if r.score >= r.min_score { 1u64 } else { 0u64 };
        request.owner.from_arcis(passes)
    }
}
