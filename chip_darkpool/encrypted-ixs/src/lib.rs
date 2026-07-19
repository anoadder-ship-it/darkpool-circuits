use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    // Chip aanbod van een verkoper
    pub struct ChipListing {
        pub chip_type:      u64,  // H100=1001 H200=1002 GB200=1003 A100=1004 MI300X=2001 Gaudi3=3001
        pub quantity:       u64,  // aantal units
        pub condition:      u64,  // 1=nieuw 2=refurb 3=gebruikt
        pub price_per_unit: u64,  // USD cents per chip
        pub delivery_days:  u64,  // levertijd in dagen
        pub region:         u64,  // 1=EU 2=US 3=Asia 4=global
        pub cert_level:     u64,  // 1=datacenter 2=workstation 3=consumer
    }

    // Kooporder van een koper
    pub struct ChipOrder {
        pub chip_type:    u64,  // gewenst chiptype
        pub min_quantity: u64,  // minimum aantal
        pub max_condition:u64,  // slechtste acceptabele conditie (hoger=soepeler)
        pub max_price:    u64,  // maximum prijs per chip (cents)
        pub max_delivery: u64,  // maximum levertijd (dagen)
        pub req_region:   u64,  // gewenste regio (4=global accepteert alles)
        pub min_cert:     u64,  // minimum certificatieniveau
    }

    // Gecombineerde matchrequest (14 velden)
    pub struct ChipMatchRequest {
        // Aanbod velden
        pub chip_type:      u64,
        pub quantity:       u64,
        pub condition:      u64,
        pub price_per_unit: u64,
        pub delivery_days:  u64,
        pub list_region:    u64,
        pub cert_level:     u64,
        // Aanvraag velden
        pub req_chip_type:  u64,
        pub min_quantity:   u64,
        pub max_condition:  u64,
        pub max_price:      u64,
        pub max_delivery:   u64,
        pub req_region:     u64,
        pub min_cert:       u64,
    }

    pub struct ChipMatchResult {
        pub matched: u64,  // 1=match 0=geen match
        pub score:   u64,  // 0-98 (7 criteria x 14 punten)
    }

    /// Registreer een versleuteld chip aanbod
    #[instruction]
    pub fn register_chip(listing: Enc<Shared, ChipListing>) -> Enc<Shared, u64> {
        let _l = listing.to_arcis();
        listing.owner.from_arcis(1u64)
    }

    /// Match chip aanbod met kooporder — volledig encrypted
    /// Zeven criteria: type exact, qty voldoende, conditie ok, prijs ok,
    ///   levering ok, regio ok, certificering ok
    #[instruction]
    pub fn match_chip(request: Enc<Shared, ChipMatchRequest>) -> Enc<Shared, ChipMatchResult> {
        let r = request.to_arcis();

        // Criterium 1: chiptype moet exact overeenkomen
        let c1 = if r.chip_type == r.req_chip_type { 1u64 } else { 0u64 };

        // Criterium 2: aanbod heeft genoeg units
        let c2 = if r.quantity >= r.min_quantity { 1u64 } else { 0u64 };

        // Criterium 3: conditie is acceptabel (lager=beter, buyer stelt max in)
        let c3 = if r.condition <= r.max_condition { 1u64 } else { 0u64 };

        // Criterium 4: prijs is binnen budget
        let c4 = if r.price_per_unit <= r.max_price { 1u64 } else { 0u64 };

        // Criterium 5: levering snel genoeg
        let c5 = if r.delivery_days <= r.max_delivery { 1u64 } else { 0u64 };

        // Criterium 6: regio klopt (4=global = accepteer altijd)
        let c6 = if r.list_region == r.req_region { 1u64 }
                 else if r.list_region == 4u64     { 1u64 }
                 else if r.req_region  == 4u64     { 1u64 }
                 else { 0u64 };

        // Criterium 7: certificering voldoende (hoger=beter)
        let c7 = if r.cert_level >= r.min_cert { 1u64 } else { 0u64 };

        // Match alleen als alle 7 criteria kloppen
        let matched = c1 * c2 * c3 * c4 * c5 * c6 * c7;

        // Score: elk criterium 14 punten (max 98)
        let score = (c1 + c2 + c3 + c4 + c5 + c6 + c7) * 14u64;

        let result = ChipMatchResult { matched, score };
        request.owner.from_arcis(result)
    }

    /// Versleutelde volumeaggregatie voor marktintelligentie
    pub struct VolumeData {
        pub chip_type: u64,
        pub volume:    u64,  // aantal chips in deze batch
        pub price:     u64,  // totale waarde (cents)
    }

    /// Aggregeer volumes van twee partijen zonder individuele data te onthullen
    #[instruction]
    pub fn aggregate_volume(data: Enc<Shared, VolumeData>) -> Enc<Shared, u64> {
        let d = data.to_arcis();
        // Retourneert totaal volume — kan later uitgebreid worden
        // voor multi-party aggregatie
        data.owner.from_arcis(d.volume)
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
