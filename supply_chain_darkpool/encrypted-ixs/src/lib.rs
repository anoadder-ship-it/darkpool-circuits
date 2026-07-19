use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct SupplyOffer {
        pub material_code: u64,
        pub quantity:      u64,
        pub quality_grade: u64,
        pub price_per_unit:u64,
        pub delivery_days: u64,
        pub region_code:   u64,
    }

    pub struct SupplyDemand {
        pub material_code:    u64,
        pub min_quantity:     u64,
        pub min_quality:      u64,
        pub max_price:        u64,
        pub max_delivery_days:u64,
        pub region_code:      u64,
    }

    pub struct MatchRequest {
        pub material_code: u64,
        pub quantity:      u64,
        pub quality_grade: u64,
        pub price_per_unit:u64,
        pub delivery_days: u64,
        pub supply_region: u64,
        pub req_material:  u64,
        pub min_quantity:  u64,
        pub min_quality:   u64,
        pub max_price:     u64,
        pub max_delivery:  u64,
        pub req_region:    u64,
    }

    pub struct MatchResult {
        pub matched: u64,
        pub score:   u64,
    }

    #[instruction]
    pub fn register_supply(offer: Enc<Shared, SupplyOffer>) -> Enc<Shared, u64> {
        let _o = offer.to_arcis();
        offer.owner.from_arcis(1u64)
    }

    #[instruction]
    pub fn match_supply(request: Enc<Shared, MatchRequest>) -> Enc<Shared, MatchResult> {
        let r = request.to_arcis();

        let material_ok  = if r.material_code == r.req_material  { 1u64 } else { 0u64 };
        let quantity_ok  = if r.quantity      >= r.min_quantity   { 1u64 } else { 0u64 };
        let quality_ok   = if r.quality_grade >= r.min_quality    { 1u64 } else { 0u64 };
        let price_ok     = if r.price_per_unit <= r.max_price     { 1u64 } else { 0u64 };
        let delivery_ok  = if r.delivery_days  <= r.max_delivery  { 1u64 } else { 0u64 };
        let region_ok    = if r.supply_region  == r.req_region    { 1u64 } else { 0u64 };

        let matched = material_ok * quantity_ok * quality_ok * price_ok * delivery_ok * region_ok;
        let score = (material_ok + quantity_ok + quality_ok + price_ok + delivery_ok + region_ok) * 16u64;

        let result = MatchResult { matched, score };
        request.owner.from_arcis(result)
    }

    pub struct CarbonOffer {
        pub credits:    u64,
        pub price:      u64,
        pub vintage:    u64,
        pub cert_type:  u64,
    }

    pub struct CarbonDemand {
        pub min_credits: u64,
        pub max_price:   u64,
        pub min_vintage: u64,
        pub cert_type:   u64,
    }

    pub struct CarbonMatch {
        pub offer_credits: u64,
        pub offer_price:   u64,
        pub offer_vintage: u64,
        pub offer_cert:    u64,
        pub req_credits:   u64,
        pub req_max_price: u64,
        pub req_vintage:   u64,
        pub req_cert:      u64,
    }

    #[instruction]
    pub fn match_carbon(request: Enc<Shared, CarbonMatch>) -> Enc<Shared, u64> {
        let r = request.to_arcis();
        let credits_ok = if r.offer_credits  >= r.req_credits  { 1u64 } else { 0u64 };
        let price_ok   = if r.offer_price    <= r.req_max_price { 1u64 } else { 0u64 };
        let vintage_ok = if r.offer_vintage  >= r.req_vintage   { 1u64 } else { 0u64 };
        let cert_ok    = if r.offer_cert     == r.req_cert      { 1u64 } else { 0u64 };
        let matched = credits_ok * price_ok * vintage_ok * cert_ok;
        request.owner.from_arcis(matched)
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
