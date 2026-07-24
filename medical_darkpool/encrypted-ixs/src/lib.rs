use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    const REGISTRY_SIZE: usize = 500;

    #[derive(Clone, Copy)]
    pub struct Dataset {
        pub disease_code:  u64,
        pub sample_count:  u64,
        pub age_mean:      u64,
        pub gender_female: u64,
        pub data_modality: u64,
        pub active:        bool,
        pub expires_at:    u64, // Unix timestamp, 0 = nooit
    }

    pub struct DatasetRegistry {
        pub datasets: [Dataset; REGISTRY_SIZE],
        pub count:    u64,
    }

    pub struct NewDataset {
        pub disease_code:  u64,
        pub sample_count:  u64,
        pub age_mean:      u64,
        pub gender_female: u64,
        pub data_modality: u64,
        pub expires_at:    u64,
    }

    pub struct SearchQuery {
        pub disease_code:  u64,
        pub min_samples:   u64,
        pub age_min:       u64,
        pub age_max:       u64,
        pub data_modality: u64,
    }

    pub struct SearchResult {
        pub best_score: u64,
        pub found:      bool,
    }

    /// Maakt een leeg, versleuteld dataset-register aan (MXE-eigendom).
    #[instruction]
    pub fn init_registry() -> Enc<Mxe, DatasetRegistry> {
        let empty = Dataset {
            disease_code:  0,
            sample_count:  0,
            age_mean:      0,
            gender_female: 0,
            data_modality: 0,
            active:        false,
            expires_at:    0,
        };
        let reg = DatasetRegistry {
            datasets: [empty; REGISTRY_SIZE],
            count:    0,
        };
        Mxe::get().from_arcis(reg)
    }

    /// Registreert een dataset in het register en onthult alleen de slot-index.
    #[instruction]
    pub fn register_dataset(
        reg_ctxt: Enc<Mxe, DatasetRegistry>,
        ds_ctxt:  Enc<Shared, NewDataset>,
    ) -> (Enc<Mxe, DatasetRegistry>, u64) {
        let mut reg = reg_ctxt.to_arcis();
        let ds      = ds_ctxt.to_arcis();
        let mut placed = false;
        let mut placed_index: u64 = REGISTRY_SIZE as u64;

        for i in 0..REGISTRY_SIZE {
            if !reg.datasets[i].active && !placed {
                reg.datasets[i].disease_code  = ds.disease_code;
                reg.datasets[i].sample_count  = ds.sample_count;
                reg.datasets[i].age_mean      = ds.age_mean;
                reg.datasets[i].gender_female = ds.gender_female;
                reg.datasets[i].data_modality = ds.data_modality;
                reg.datasets[i].active        = true;
                reg.datasets[i].expires_at    = ds.expires_at;
                reg.count += 1;
                placed = true;
                placed_index = i as u64;
            }
        }

        (Mxe::get().from_arcis(reg), placed_index.reveal())
    }

    /// Doorzoekt het HELE register op de best passende dataset voor de
    /// query (O(n), score-prioriteit), slaat verlopen datasets over.
    /// Onthult de index van de beste match plus (versleuteld) de score.
    #[instruction]
    pub fn search_datasets(
        reg_ctxt:     Enc<Mxe, DatasetRegistry>,
        query_ctxt:   Enc<Shared, SearchQuery>,
        current_time: u64,
    ) -> (Enc<Shared, SearchResult>, u64) {
        let reg   = reg_ctxt.to_arcis();
        let query = query_ctxt.to_arcis();

        let mut best_score: u64 = 0;
        let mut best_idx:   u64 = REGISTRY_SIZE as u64;
        let mut found = false;

        for i in 0..REGISTRY_SIZE {
            let expired = reg.datasets[i].expires_at > 0 && reg.datasets[i].expires_at < current_time;
            if reg.datasets[i].active && !expired {
                let disease_match = if reg.datasets[i].disease_code == query.disease_code { 1u64 } else { 0u64 };
                let size_ok  = if reg.datasets[i].sample_count >= query.min_samples { 1u64 } else { 0u64 };
                let age_ok   = if reg.datasets[i].age_mean >= query.age_min && reg.datasets[i].age_mean <= query.age_max { 1u64 } else { 0u64 };
                let modality_ok = if reg.datasets[i].data_modality == query.data_modality { 1u64 } else { 0u64 };
                let compatible = disease_match * size_ok * age_ok * modality_ok;
                let score = (disease_match + size_ok + age_ok + modality_ok) * 25u64 * compatible;
                if compatible == 1 && score > best_score {
                    best_score = score;
                    best_idx   = i as u64;
                    found      = true;
                }
            }
        }

        let result = SearchResult { best_score, found };
        (query_ctxt.owner.from_arcis(result), best_idx.reveal())
    }

    /// Verwijdert de dataset op de gegeven index (eigendom on-chain gecheckt).
    #[instruction]
    pub fn remove_dataset(
        reg_ctxt: Enc<Mxe, DatasetRegistry>,
        index:    u64,
    ) -> Enc<Mxe, DatasetRegistry> {
        let mut reg = reg_ctxt.to_arcis();

        for i in 0..REGISTRY_SIZE {
            if (i as u64) == index {
                reg.datasets[i].active        = false;
                reg.datasets[i].disease_code  = 0;
                reg.datasets[i].sample_count  = 0;
                if reg.count > 0 {
                    reg.count -= 1;
                }
            }
        }

        Mxe::get().from_arcis(reg)
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
