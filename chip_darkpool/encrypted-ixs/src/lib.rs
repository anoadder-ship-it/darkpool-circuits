use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    const CHIP_BOOK_SIZE: usize = 500;

    #[derive(Clone, Copy)]
    pub struct ChipOffer {
        pub chip_type:  u64,  // categorie (GPU/CPU/ASIC/FPGA/...)
        pub volume:     u64,  // aantal chips
        pub unit_price: u64,
        pub is_supply:  bool, // true = aanbod, false = vraag
        pub active:     bool,
        pub expires_at: u64,  // Unix timestamp, 0 = nooit
    }

    pub struct ChipBook {
        pub offers: [ChipOffer; CHIP_BOOK_SIZE],
        pub count:  u64,
    }

    pub struct NewChipOffer {
        pub chip_type:  u64,
        pub volume:     u64,
        pub unit_price: u64,
        pub is_supply:  bool,
        pub expires_at: u64,
    }

    pub struct ChipMatchResult {
        pub chip_type:     u64,
        pub matched_vol:   u64,
        pub matched_price: u64,
        pub matched:       bool,
    }

    pub struct ChipStats {
        pub total_offers:  u64,
        pub supply_volume: u64,
        pub demand_volume: u64,
    }

    /// Maakt een leeg, versleuteld chip-boek aan (MXE-eigendom).
    #[instruction]
    pub fn init_chip_book() -> Enc<Mxe, ChipBook> {
        let empty = ChipOffer {
            chip_type:  0,
            volume:     0,
            unit_price: 0,
            is_supply:  false,
            active:     false,
            expires_at: 0,
        };
        let book = ChipBook {
            offers: [empty; CHIP_BOOK_SIZE],
            count:  0,
        };
        Mxe::get().from_arcis(book)
    }

    /// Registreert een chip-aanbod/vraag en onthult alleen de slot-index.
    #[instruction]
    pub fn register_chip(
        book_ctxt:  Enc<Mxe, ChipBook>,
        offer_ctxt: Enc<Shared, NewChipOffer>,
    ) -> (Enc<Mxe, ChipBook>, u64) {
        let mut book   = book_ctxt.to_arcis();
        let offer      = offer_ctxt.to_arcis();
        let mut placed = false;
        let mut placed_index: u64 = CHIP_BOOK_SIZE as u64;

        for i in 0..CHIP_BOOK_SIZE {
            if !book.offers[i].active && !placed {
                book.offers[i].chip_type  = offer.chip_type;
                book.offers[i].volume     = offer.volume;
                book.offers[i].unit_price = offer.unit_price;
                book.offers[i].is_supply  = offer.is_supply;
                book.offers[i].active     = true;
                book.offers[i].expires_at = offer.expires_at;
                book.count += 1;
                placed = true;
                placed_index = i as u64;
            }
        }

        (Mxe::get().from_arcis(book), placed_index.reveal())
    }

    /// Vindt beste aanbod (laagste prijs) en beste vraag (hoogste prijs)
    /// voor dit chiptype, O(n), slaat verlopen aanbiedingen over.
    #[instruction]
    pub fn match_chip(
        book_ctxt:    Enc<Mxe, ChipBook>,
        type_ctxt:    Enc<Shared, u64>,
        current_time: u64,
    ) -> (Enc<Shared, ChipMatchResult>, u64, u64) {
        let book        = book_ctxt.to_arcis();
        let target_type = type_ctxt.to_arcis();

        let mut has_supply    = false;
        let mut supply_price: u64 = 0;
        let mut supply_vol:   u64 = 0;
        let mut supply_idx:   u64 = CHIP_BOOK_SIZE as u64;

        let mut has_demand    = false;
        let mut demand_price: u64 = 0;
        let mut demand_vol:   u64 = 0;
        let mut demand_idx:   u64 = CHIP_BOOK_SIZE as u64;

        for i in 0..CHIP_BOOK_SIZE {
            let expired = book.offers[i].expires_at > 0 && book.offers[i].expires_at < current_time;
            if book.offers[i].active && !expired && book.offers[i].chip_type == target_type {
                if book.offers[i].is_supply {
                    if !has_supply || book.offers[i].unit_price < supply_price {
                        has_supply   = true;
                        supply_price = book.offers[i].unit_price;
                        supply_vol   = book.offers[i].volume;
                        supply_idx   = i as u64;
                    }
                } else {
                    if !has_demand || book.offers[i].unit_price > demand_price {
                        has_demand   = true;
                        demand_price = book.offers[i].unit_price;
                        demand_vol   = book.offers[i].volume;
                        demand_idx   = i as u64;
                    }
                }
            }
        }

        let mut result = ChipMatchResult {
            chip_type:     target_type,
            matched_vol:   0,
            matched_price: 0,
            matched:       false,
        };
        let mut out_supply_idx = CHIP_BOOK_SIZE as u64;
        let mut out_demand_idx = CHIP_BOOK_SIZE as u64;

        if has_supply && has_demand && demand_price >= supply_price {
            result.matched_vol   = if supply_vol < demand_vol { supply_vol } else { demand_vol };
            result.matched_price = (supply_price + demand_price) / 2;
            result.matched       = true;
            out_supply_idx = supply_idx;
            out_demand_idx = demand_idx;
        }

        (type_ctxt.owner.from_arcis(result), out_supply_idx.reveal(), out_demand_idx.reveal())
    }

    /// Partial fills op volumes: vermindert beide kanten met het gevulde
    /// volume; alleen volledig gevulde aanbiedingen worden inactief.
    #[instruction]
    pub fn settle_chip(
        book_ctxt:  Enc<Mxe, ChipBook>,
        supply_idx: u64,
        demand_idx: u64,
    ) -> Enc<Mxe, ChipBook> {
        let mut book = book_ctxt.to_arcis();

        let mut supply_vol: u64 = 0;
        let mut demand_vol: u64 = 0;

        for i in 0..CHIP_BOOK_SIZE {
            if (i as u64) == supply_idx {
                supply_vol = book.offers[i].volume;
            }
            if (i as u64) == demand_idx {
                demand_vol = book.offers[i].volume;
            }
        }

        let fill_vol = if supply_vol < demand_vol { supply_vol } else { demand_vol };

        for i in 0..CHIP_BOOK_SIZE {
            if (i as u64) == supply_idx {
                book.offers[i].volume = book.offers[i].volume - fill_vol;
                if book.offers[i].volume == 0 {
                    book.offers[i].active     = false;
                    book.offers[i].unit_price = 0;
                    if book.count > 0 {
                        book.count -= 1;
                    }
                }
            }
            if (i as u64) == demand_idx {
                book.offers[i].volume = book.offers[i].volume - fill_vol;
                if book.offers[i].volume == 0 {
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

    /// Annuleert het aanbod op de gegeven index (eigendom on-chain gecheckt).
    #[instruction]
    pub fn cancel_chip(
        book_ctxt: Enc<Mxe, ChipBook>,
        index:     u64,
    ) -> Enc<Mxe, ChipBook> {
        let mut book = book_ctxt.to_arcis();

        for i in 0..CHIP_BOOK_SIZE {
            if (i as u64) == index {
                book.offers[i].active     = false;
                book.offers[i].volume     = 0;
                book.offers[i].unit_price = 0;
                if book.count > 0 {
                    book.count -= 1;
                }
            }
        }

        Mxe::get().from_arcis(book)
    }

    #[instruction]
    pub fn aggregate_volume(
        book_ctxt: Enc<Mxe, ChipBook>,
    ) -> Enc<Mxe, ChipStats> {
        let book = book_ctxt.to_arcis();

        let mut stats = ChipStats {
            total_offers:  0,
            supply_volume: 0,
            demand_volume: 0,
        };

        for i in 0..CHIP_BOOK_SIZE {
            if book.offers[i].active {
                stats.total_offers += 1;
                if book.offers[i].is_supply {
                    stats.supply_volume += book.offers[i].volume;
                } else {
                    stats.demand_volume += book.offers[i].volume;
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
