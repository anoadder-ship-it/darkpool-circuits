use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    const ORDER_BOOK_SIZE: usize = 1000;

    #[derive(Clone, Copy)]
    pub struct Order {
        pub asset_id: u64,
        pub bid:      u64,
        pub size:     u64,
        pub is_buy:   bool,
        pub active:   bool,
    }

    pub struct OrderBook {
        pub orders: [Order; ORDER_BOOK_SIZE],
        pub count:  u64,
    }

    pub struct NewOrder {
        pub asset_id: u64,
        pub bid:      u64,
        pub size:     u64,
        pub is_buy:   bool,
    }

    pub struct MatchResult {
        pub asset_id:      u64,
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
    pub fn init_order_book() -> Enc<Mxe, OrderBook> {
        let empty_order = Order {
            asset_id: 0,
            bid:      0,
            size:     0,
            is_buy:   false,
            active:   false,
        };
        let book = OrderBook {
            orders: [empty_order; ORDER_BOOK_SIZE],
            count:  0,
        };
        Mxe::get().from_arcis(book)
    }

    #[instruction]
    pub fn place_order(
        book_ctxt:  Enc<Mxe, OrderBook>,
        order_ctxt: Enc<Shared, NewOrder>,
    ) -> (Enc<Mxe, OrderBook>, u64) {
        let mut book    = book_ctxt.to_arcis();
        let new_order   = order_ctxt.to_arcis();
        let mut placed  = false;
        let mut placed_index: u64 = ORDER_BOOK_SIZE as u64;

        for i in 0..ORDER_BOOK_SIZE {
            if !book.orders[i].active && !placed {
                book.orders[i].asset_id = new_order.asset_id;
                book.orders[i].bid      = new_order.bid;
                book.orders[i].size     = new_order.size;
                book.orders[i].is_buy   = new_order.is_buy;
                book.orders[i].active   = true;
                book.count += 1;
                placed = true;
                placed_index = i as u64;
            }
        }

        (Mxe::get().from_arcis(book), placed_index.reveal())
    }

    #[instruction]
    pub fn match_orders(
        book_ctxt:  Enc<Mxe, OrderBook>,
        asset_ctxt: Enc<Shared, u64>,
    ) -> (Enc<Shared, MatchResult>, u64, u64) {
        let book         = book_ctxt.to_arcis();
        let target_asset = asset_ctxt.to_arcis();

        let mut has_buy    = false;
        let mut buy_bid:   u64 = 0;
        let mut buy_size:  u64 = 0;
        let mut buy_idx:   u64 = ORDER_BOOK_SIZE as u64;

        let mut has_sell   = false;
        let mut sell_bid:  u64 = 0;
        let mut sell_size: u64 = 0;
        let mut sell_idx:  u64 = ORDER_BOOK_SIZE as u64;

        for i in 0..ORDER_BOOK_SIZE {
            if book.orders[i].active && book.orders[i].asset_id == target_asset {
                if book.orders[i].is_buy {
                    if !has_buy || book.orders[i].bid > buy_bid {
                        has_buy  = true;
                        buy_bid  = book.orders[i].bid;
                        buy_size = book.orders[i].size;
                        buy_idx  = i as u64;
                    }
                } else {
                    if !has_sell || book.orders[i].bid < sell_bid {
                        has_sell  = true;
                        sell_bid  = book.orders[i].bid;
                        sell_size = book.orders[i].size;
                        sell_idx  = i as u64;
                    }
                }
            }
        }

        let mut result = MatchResult {
            asset_id:      target_asset,
            matched_size:  0,
            matched_price: 0,
            matched:       false,
        };
        let mut out_buy_idx  = ORDER_BOOK_SIZE as u64;
        let mut out_sell_idx = ORDER_BOOK_SIZE as u64;

        if has_buy && has_sell && buy_bid >= sell_bid {
            result.matched_size  = if buy_size < sell_size { buy_size } else { sell_size };
            result.matched_price = (buy_bid + sell_bid) / 2;
            result.matched        = true;
            out_buy_idx  = buy_idx;
            out_sell_idx = sell_idx;
        }

        (asset_ctxt.owner.from_arcis(result), out_buy_idx.reveal(), out_sell_idx.reveal())
    }

    #[instruction]
    pub fn settle_match(
        book_ctxt: Enc<Mxe, OrderBook>,
        buy_idx:   u64,
        sell_idx:  u64,
    ) -> Enc<Mxe, OrderBook> {
        let mut book = book_ctxt.to_arcis();

        for i in 0..ORDER_BOOK_SIZE {
            if (i as u64) == buy_idx || (i as u64) == sell_idx {
                book.orders[i].active = false;
                book.orders[i].bid    = 0;
                book.orders[i].size   = 0;
                if book.count > 0 {
                    book.count -= 1;
                }
            }
        }

        Mxe::get().from_arcis(book)
    }

    #[instruction]
    pub fn cancel_trade_order(
        book_ctxt: Enc<Mxe, OrderBook>,
        index:     u64,
    ) -> Enc<Mxe, OrderBook> {
        let mut book = book_ctxt.to_arcis();

        for i in 0..ORDER_BOOK_SIZE {
            if (i as u64) == index {
                book.orders[i].active = false;
                book.orders[i].bid    = 0;
                book.orders[i].size   = 0;
                if book.count > 0 {
                    book.count -= 1;
                }
            }
        }

        Mxe::get().from_arcis(book)
    }

    #[instruction]
    pub fn get_stats(
        book_ctxt: Enc<Mxe, OrderBook>,
    ) -> Enc<Mxe, BookStats> {
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

        Mxe::get().from_arcis(stats)
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
