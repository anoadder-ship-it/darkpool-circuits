use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct DatasetProfile {
        pub disease_code:   u64,
        pub sample_count:   u64,
        pub age_mean:       u64,
        pub gender_female:  u64,
        pub data_modality:  u64,
    }

    pub struct SearchQuery {
        pub disease_code:   u64,
        pub min_samples:    u64,
        pub age_min:        u64,
        pub age_max:        u64,
        pub data_modality:  u64,
    }

    pub struct MatchRequest {
        pub disease_code:   u64,
        pub sample_count:   u64,
        pub age_mean:       u64,
        pub gender_female:  u64,
        pub data_modality:  u64,
        pub query_disease:  u64,
        pub min_samples:    u64,
        pub age_min:        u64,
        pub age_max:        u64,
        pub query_modality: u64,
    }

    pub struct CompatibilityResult {
        pub compatible: u64,
        pub score:      u64,
    }

    #[instruction]
    pub fn register_dataset(profile: Enc<Shared, DatasetProfile>) -> Enc<Shared, u64> {
        let _p = profile.to_arcis();
        profile.owner.from_arcis(1u64)
    }

    #[instruction]
    pub fn match_dataset(request: Enc<Shared, MatchRequest>) -> Enc<Shared, CompatibilityResult> {
        let r = request.to_arcis();
        let disease_match = if r.disease_code == r.query_disease { 1u64 } else { 0u64 };
        let size_ok = if r.sample_count >= r.min_samples { 1u64 } else { 0u64 };
        let age_ok = if r.age_mean >= r.age_min && r.age_mean <= r.age_max { 1u64 } else { 0u64 };
        let modality_ok = if r.data_modality == r.query_modality { 1u64 } else { 0u64 };
        let compatible = disease_match * size_ok * age_ok * modality_ok;
        let score = (disease_match + size_ok + age_ok + modality_ok) * 25u64;
        let result = CompatibilityResult { compatible, score };
        request.owner.from_arcis(result)
    }

    #[instruction]
    pub fn aggregate_gradient(gradient: Enc<Shared, u64>) -> Enc<Shared, u64> {
        let g = gradient.to_arcis();
        gradient.owner.from_arcis(g)
    }
}
