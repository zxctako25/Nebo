use super::versions::*;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub struct ProtocolRegistry;

static INSTANCE: OnceLock<Registry> = OnceLock::new();

struct Registry {
    by_protocol: HashMap<i32, Arc<dyn MinecraftProtocol>>,
    fallback: Arc<dyn MinecraftProtocol>,
}

impl ProtocolRegistry {
    fn get() -> &'static Registry {
        INSTANCE.get_or_init(|| {
            let p16 = Arc::new(Protocol1_16_5) as Arc<dyn MinecraftProtocol>;
            let p17 = Arc::new(Protocol1_17) as Arc<dyn MinecraftProtocol>;
            let p18 = Arc::new(Protocol1_18) as Arc<dyn MinecraftProtocol>;
            let p19 = Arc::new(Protocol1_19) as Arc<dyn MinecraftProtocol>;
            let p19_3 = Arc::new(Protocol1_19_3) as Arc<dyn MinecraftProtocol>;
            let p20 = Arc::new(Protocol1_20) as Arc<dyn MinecraftProtocol>;
            let p20_2 = Arc::new(Protocol1_20_2) as Arc<dyn MinecraftProtocol>;
            let p21_11 = Arc::new(Protocol1_21_11) as Arc<dyn MinecraftProtocol>;

            let mut m = HashMap::new();
            m.insert(754, p16.clone());
            m.insert(755, p17.clone());
            m.insert(756, p17);
            m.insert(757, p18.clone());
            m.insert(758, p18);
            m.insert(759, p19.clone());
            m.insert(760, p19);
            m.insert(761, p19_3.clone());
            m.insert(762, p19_3);
            m.insert(763, p20);
            for id in 764..=773 {
                m.insert(id, p20_2.clone());
            }
            m.insert(774, p21_11.clone());

            Registry {
                by_protocol: m,
                fallback: p16,
            }
        })
    }

    pub fn by_protocol(protocol: i32) -> Arc<dyn MinecraftProtocol> {
        Self::get()
            .by_protocol
            .get(&protocol)
            .cloned()
            .unwrap_or_else(|| Self::get().fallback.clone())
    }

    pub fn fallback() -> Arc<dyn MinecraftProtocol> {
        Self::get().fallback.clone()
    }

    pub fn is_supported(protocol: i32) -> bool {
        Self::get().by_protocol.contains_key(&protocol)
    }

    pub fn supported_list() -> &'static str {
        "1.16.5(754), 1.17(755), 1.17.1(756), 1.18(757), 1.18.2(758), 1.19(759), 1.19.1(760), 1.19.3(761), 1.19.4(762), 1.20(763), 1.20.2-1.21.9(764-773), 1.21.11(774)"
    }
}
