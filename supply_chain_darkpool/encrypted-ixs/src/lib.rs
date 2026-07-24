use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    const SUPPLY_BOOK_SIZE: usize = 500;

    #[derive(Clone, Copy)]
    pub struct SupplyOffer {
        pub material_id: u64,
        pub quantity:    u64,
        pub unit_price:  u64,
        pub is_supply:   bool,  // true = aanbod, false = vraag
        pub active:      bool,
        pub expires_at:  u64,   // Unix timestamp, 0 = nooit
    }

    pub struct SupplyBook {
        pub offers: [SupplyOffer; SUPPLY_BOOK_SIZE],
        pub count:  u64,
    }

    pub struct NewSupplyOffer {
        pub material_id: u64,
        pub quantity:    u64,
        pub unit_price:  u64,
        pub is_supply:   bool,
        pub expires_at:  u64,
    }

    pub struct SupplyMatchResult {
        pub material_id:   u64,
        pub matched_qty:   u64,
        pub matched_price: u64,
        pub matched:       bool,
    }

    pub struct SupplyStats {
        pub total_offers:  u64,
        pub supply_volume: u64,
        pub demand_volume: u64,
    }

    /// Maakt een leeg, versleuteld supply-boek aan (MXE-eigendom).
    #[instruction]
    pub fn init_supply_book() -> Enc<Mxe, SupplyBook> {
        let empty = SupplyOffer {
            material_id: 0,
            quantity:    0,
            unit_price:  0,
            is_supply:   false,
            active:      false,
            expires_at:  0,
        };
        let book = SupplyBook {
            offers: [empty; SUPPLY_BOOK_SIZE],
            count:  0,
        };
        Mxe::get().from_arcis(book)
    }

    /// Plaatst een aanbod/vraag en onthult alleen de slot-index.
    #[instruction]
    pub fn register_supply(
        book_ctxt:  Enc<Mxe, SupplyBook>,
        offer_ctxt: Enc<Shared, NewSupplyOffer>,
    ) -> (Enc<Mxe, SupplyBook>, u64) {
        let mut book  = book_ctxt.to_arcis();
        let offer     = offer_ctxt.to_arcis();
        let mut placed = false;
        let mut placed_index: u64 = SUPPLY_BOOK_SIZE as u64;

        for i in 0..SUPPLY_BOOK_SIZE {
            if !book.offers[i].active && !placed {
                book.offers[i].material_id = offer.material_id;
                book.offers[i].quantity    = offer.quantity;
                book.offers[i].unit_price  = offer.unit_price;
                book.offers[i].is_supply   = offer.is_supply;
                book.offers[i].active      = true;
                book.offers[i].expires_at  = offer.expires_at;
                book.count += 1;
                placed = true;
                placed_index = i as u64;
            }
        }

        (Mxe::get().from_arcis(book), placed_index.reveal())
    }

    /// Vindt beste aanbod (laagste prijs) en beste vraag (hoogste prijs)
    /// voor dit materiaal, O(n), slaat verlopen aanbiedingen over.
    #[instruction]
    pub fn match_supply(
        book_ctxt:    Enc<Mxe, SupplyBook>,
        mat_ctxt:     Enc<Shared, u64>,
        current_time: u64,
    ) -> (Enc<Shared, SupplyMatchResult>, u64, u64) {
        let book       = book_ctxt.to_arcis();
        let target_mat = mat_ctxt.to_arcis();

        let mut has_supply    = false;
        let mut supply_price: u64 = 0;
        let mut supply_qty:   u64 = 0;
        let mut supply_idx:   u64 = SUPPLY_BOOK_SIZE as u64;

        let mut has_demand    = false;
        let mut demand_price: u64 = 0;
        let mut demand_qty:   u64 = 0;
        let mut demand_idx:   u64 = SUPPLY_BOOK_SIZE as u64;

        for i in 0..SUPPLY_BOOK_SIZE {
            let expired = book.offers[i].expires_at > 0 && book.offers[i].expires_at < current_time;
            if book.offers[i].active && !expired && book.offers[i].material_id == target_mat {
                if book.offers[i].is_supply {
                    if !has_supply || book.offers[i].unit_price < supply_price {
                        has_supply   = true;
                        supply_price = book.offers[i].unit_price;
                        supply_qty   = book.offers[i].quantity;
                        supply_idx   = i as u64;
                    }
                } else {
                    if !has_demand || book.offers[i].unit_price > demand_price {
                        has_demand   = true;
                        demand_price = book.offers[i].unit_price;
                        demand_qty   = book.offers[i].quantity;
                        demand_idx   = i as u64;
                    }
                }
            }
        }

        let mut result = SupplyMatchResult {
            material_id:   target_mat,
            matched_qty:   0,
            matched_price: 0,
            matched:       false,
        };
        let mut out_supply_idx = SUPPLY_BOOK_SIZE as u64;
        let mut out_demand_idx = SUPPLY_BOOK_SIZE as u64;

        if has_supply && has_demand && demand_price >= supply_price {
            result.matched_qty   = if supply_qty < demand_qty { supply_qty } else { demand_qty };
            result.matched_price = (supply_price + demand_price) / 2;
            result.matched       = true;
            out_supply_idx = supply_idx;
            out_demand_idx = demand_idx;
        }

        (mat_ctxt.owner.from_arcis(result), out_supply_idx.reveal(), out_demand_idx.reveal())
    }

    /// Partial fulfillment: vermindert beide kanten met de gevulde
    /// hoeveelheid; alleen volledig gevulde aanbiedingen worden inactief.
    #[instruction]
    pub fn settle_supply(
        book_ctxt:  Enc<Mxe, SupplyBook>,
        supply_idx: u64,
        demand_idx: u64,
    ) -> Enc<Mxe, SupplyBook> {
        let mut book = book_ctxt.to_arcis();

        let mut supply_qty: u64 = 0;
        let mut demand_qty: u64 = 0;

        for i in 0..SUPPLY_BOOK_SIZE {
            if (i as u64) == supply_idx {
                supply_qty = book.offers[i].quantity;
            }
            if (i as u64) == demand_idx {
                demand_qty = book.offers[i].quantity;
            }
        }

        let fill_qty = if supply_qty < demand_qty { supply_qty } else { demand_qty };

        for i in 0..SUPPLY_BOOK_SIZE {
            if (i as u64) == supply_idx {
                book.offers[i].quantity = book.offers[i].quantity - fill_qty;
                if book.offers[i].quantity == 0 {
                    book.offers[i].active     = false;
                    book.offers[i].unit_price = 0;
                    if book.count > 0 {
                        book.count -= 1;
                    }
                }
            }
            if (i as u64) == demand_idx {
                book.offers[i].quantity = book.offers[i].quantity - fill_qty;
                if book.offers[i].quantity == 0 {
                    book.offers[i].active     = false;
                    book.offers[i].unit_price = 0;
                    if book.count > 0 {
                        book.count -= 1;
                    }
                }
            }
        }

        Mxe::get().from_arcis(book)
    }

    /// Annuleert de aanbieding op de gegeven index (eigendom on-chain gecheckt).
    #[instruction]
    pub fn cancel_supply(
        book_ctxt: Enc<Mxe, SupplyBook>,
        index:     u64,
    ) -> Enc<Mxe, SupplyBook> {
        let mut book = book_ctxt.to_arcis();

        for i in 0..SUPPLY_BOOK_SIZE {
            if (i as u64) == index {
                book.offers[i].active      = false;
                book.offers[i].quantity    = 0;
                book.offers[i].unit_price  = 0;
                if book.count > 0 {
                    book.count -= 1;
                }
            }
        }

        Mxe::get().from_arcis(book)
    }

    #[instruction]
    pub fn get_supply_stats(
        book_ctxt: Enc<Mxe, SupplyBook>,
    ) -> Enc<Mxe, SupplyStats> {
        let book = book_ctxt.to_arcis();

        let mut stats = SupplyStats {
            total_offers:  0,
            supply_volume: 0,
            demand_volume: 0,
        };

        for i in 0..SUPPLY_BOOK_SIZE {
            if book.offers[i].active {
                stats.total_offers += 1;
                if book.offers[i].is_supply {
                    stats.supply_volume += book.offers[i].quantity;
                } else {
                    stats.demand_volume += book.offers[i].quantity;
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
