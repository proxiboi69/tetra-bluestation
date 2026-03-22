use std::collections::{HashMap, HashSet};
use tetra_core::TimeslotAllocator;

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub issi: u32,
    // Set of attached GSSIs
    pub attached_groups: HashSet<u32>,
}

/// Centralized subscriber registry tracking locally registered ISSIs and their group affiliations.
#[derive(Debug, Clone)]
pub struct SubscriberRegistry {
    /// Registered ISSIs → Subscriber information
    subscribers: HashMap<u32, Subscriber>,
    /// Set of all GSSIs with at least one local affiliate
    all_attached_groups: HashSet<u32>,
    /// DMO gateway: DM-MS SSI → gateway ISSI that registered it
    dm_ms_to_gateway: HashMap<u32, u32>,
    /// DMO gateway: gateway ISSI → set of DM-MS SSIs it serves
    gateway_dm_ms: HashMap<u32, HashSet<u32>>,
}

impl SubscriberRegistry {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            all_attached_groups: HashSet::new(),
            dm_ms_to_gateway: HashMap::new(),
            gateway_dm_ms: HashMap::new(),
        }
    }

    pub fn is_registered(&self, issi: u32) -> bool {
        self.subscribers.contains_key(&issi)
    }

    /// Tolerant registration; if ISSI already registered, we overwrite it with a fresh Subscriber struct
    pub fn register(&mut self, issi: u32) {
        self.deregister(issi); // Clean up any existing registration to prevent stale affiliations
        self.subscribers.insert(
            issi,
            Subscriber {
                issi,
                attached_groups: HashSet::new(),
            },
        );
    }

    /// Gets mutable ref to subscriber. If not registered, a default Subscriber is inserted.
    pub fn get_subscriber_mut(&mut self, issi: u32) -> &mut Subscriber {
        self.subscribers.entry(issi).or_insert_with(|| Subscriber {
            issi,
            attached_groups: HashSet::new(),
        })
    }

    /// Deregister an ISSI, removing it from the registry and cleaning up any group affiliations
    /// and gateway state.
    pub fn deregister(&mut self, issi: u32) {
        if let Some(subscriber) = self.subscribers.remove(&issi) {
            // Clean up global group affiliations for this subscriber
            for gssi in &subscriber.attached_groups {
                // Check if any other subscriber is still affiliated with this group
                let still_has_members = self.subscribers.values().any(|s| s.attached_groups.contains(gssi));
                if !still_has_members {
                    self.all_attached_groups.remove(gssi);
                }
            }
        }
        // Clean up any gateway state for this ISSI
        self.deregister_gateway(issi);
    }

    /// Add GSSI to subscriber's attached groups and global set
    pub fn affiliate(&mut self, issi: u32, gssi: u32) {
        let subscriber = self.get_subscriber_mut(issi);
        subscriber.attached_groups.insert(gssi);
        self.all_attached_groups.insert(gssi);
    }

    /// Remove GSSI from subscriber's attached groups. Update global set if no more subscribers are affiliated with this GSSI.
    pub fn deaffiliate(&mut self, issi: u32, gssi: u32) {
        let subscriber = self.get_subscriber_mut(issi);
        if subscriber.attached_groups.remove(&gssi) {
            // Check if any other subscriber is still affiliated with this group
            let still_has_members = self.subscribers.values().any(|s| s.attached_groups.contains(&gssi));
            if !still_has_members {
                self.all_attached_groups.remove(&gssi);
            }
        }
    }

    /// Check if any subscriber is affiliated with the given GSSI
    pub fn has_group_members(&self, gssi: u32) -> bool {
        self.all_attached_groups.contains(&gssi)
    }

    // --- DMO Gateway support (EN 300 396-5) ---

    /// Register an ISSI as a DMO gateway with the given DM-MS address set.
    pub fn register_gateway(&mut self, gateway_issi: u32, dm_ms_ssis: Vec<u32>) {
        // Clean any previous gateway state for this ISSI
        self.deregister_gateway(gateway_issi);
        let mut set = HashSet::new();
        for ssi in dm_ms_ssis {
            self.dm_ms_to_gateway.insert(ssi, gateway_issi);
            set.insert(ssi);
        }
        self.gateway_dm_ms.insert(gateway_issi, set);
    }

    /// Remove gateway status and all DM-MS address mappings for this ISSI.
    pub fn deregister_gateway(&mut self, gateway_issi: u32) {
        if let Some(dm_ms_set) = self.gateway_dm_ms.remove(&gateway_issi) {
            for ssi in dm_ms_set {
                self.dm_ms_to_gateway.remove(&ssi);
            }
        }
    }

    /// Add DM-MS addresses to an existing gateway.
    pub fn add_dm_ms_addresses(&mut self, gateway_issi: u32, dm_ms_ssis: Vec<u32>) {
        let set = self.gateway_dm_ms.entry(gateway_issi).or_default();
        for ssi in dm_ms_ssis {
            self.dm_ms_to_gateway.insert(ssi, gateway_issi);
            set.insert(ssi);
        }
    }

    /// Remove DM-MS addresses from an existing gateway.
    pub fn remove_dm_ms_addresses(&mut self, gateway_issi: u32, dm_ms_ssis: Vec<u32>) {
        if let Some(set) = self.gateway_dm_ms.get_mut(&gateway_issi) {
            for ssi in dm_ms_ssis {
                set.remove(&ssi);
                self.dm_ms_to_gateway.remove(&ssi);
            }
        }
    }

    /// Replace all DM-MS addresses for a gateway with a new set.
    pub fn replace_dm_ms_addresses(&mut self, gateway_issi: u32, dm_ms_ssis: Vec<u32>) {
        self.deregister_gateway(gateway_issi);
        self.register_gateway(gateway_issi, dm_ms_ssis);
    }

    /// Check if a given SSI is a DM-MS reachable through a gateway. Returns gateway ISSI if so.
    pub fn find_gateway_for_dm_ms(&self, dm_ms_ssi: u32) -> Option<u32> {
        self.dm_ms_to_gateway.get(&dm_ms_ssi).copied()
    }

    /// Check if a given ISSI is operating as a gateway.
    pub fn is_gateway(&self, issi: u32) -> bool {
        self.gateway_dm_ms.contains_key(&issi)
    }
}

/// Mutable, stack-editable state (mutex-protected).
#[derive(Debug, Clone)]
pub struct StackState {
    pub timeslot_alloc: TimeslotAllocator,
    /// Backhaul/network connection to SwMI (e.g., Brew/TetraPack). False -> fallback mode.
    pub network_connected: bool,
    /// Centralized subscriber registry for local-first routing decisions.
    pub subscribers: SubscriberRegistry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_deregister() {
        let mut reg = SubscriberRegistry::new();
        assert!(!reg.is_registered(1001));
        reg.register(1001);
        assert!(reg.is_registered(1001));
        reg.deregister(1001);
        assert!(!reg.is_registered(1001));
    }

    #[test]
    fn test_affiliate_deaffiliate() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.affiliate(1001, 91);
        assert!(reg.has_group_members(91));
        reg.deaffiliate(1001, 91);
        assert!(!reg.has_group_members(91));
    }

    #[test]
    fn test_has_group_members() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.register(1002);
        reg.register(1003);
        reg.affiliate(1001, 100);
        reg.affiliate(1002, 100);
        reg.affiliate(1003, 100);
        assert!(reg.has_group_members(100));

        // Deaffiliate one, should still have members
        reg.deaffiliate(1001, 100);
        assert!(reg.has_group_members(100));

        // Deregister a user, should still have members
        reg.deregister(1002);
        assert!(reg.has_group_members(100));

        // Deregister last user, should have no members
        reg.deregister(1003);
        assert!(!reg.has_group_members(100));
    }

    #[test]
    fn test_has_group_members_empty() {
        let reg = SubscriberRegistry::new();
        assert!(!reg.has_group_members(999));
    }

    #[test]
    fn test_gateway_register_deregister() {
        let mut reg = SubscriberRegistry::new();
        reg.register(2001);
        reg.register_gateway(2001, vec![3001, 3002, 3003]);

        assert!(reg.is_gateway(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3001), Some(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3002), Some(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3003), Some(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3004), None);

        reg.deregister_gateway(2001);
        assert!(!reg.is_gateway(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3001), None);
    }

    #[test]
    fn test_gateway_add_remove_addresses() {
        let mut reg = SubscriberRegistry::new();
        reg.register(2001);
        reg.register_gateway(2001, vec![3001]);

        reg.add_dm_ms_addresses(2001, vec![3002, 3003]);
        assert_eq!(reg.find_gateway_for_dm_ms(3002), Some(2001));

        reg.remove_dm_ms_addresses(2001, vec![3001]);
        assert_eq!(reg.find_gateway_for_dm_ms(3001), None);
        assert_eq!(reg.find_gateway_for_dm_ms(3002), Some(2001));
    }

    #[test]
    fn test_gateway_replace_addresses() {
        let mut reg = SubscriberRegistry::new();
        reg.register(2001);
        reg.register_gateway(2001, vec![3001, 3002]);
        reg.replace_dm_ms_addresses(2001, vec![3003, 3004]);

        assert_eq!(reg.find_gateway_for_dm_ms(3001), None);
        assert_eq!(reg.find_gateway_for_dm_ms(3003), Some(2001));
    }

    #[test]
    fn test_gateway_cleanup_on_deregister() {
        let mut reg = SubscriberRegistry::new();
        reg.register(2001);
        reg.register_gateway(2001, vec![3001, 3002]);
        // Deregistering the subscriber should also clean up gateway state
        reg.deregister(2001);
        assert!(!reg.is_gateway(2001));
        assert_eq!(reg.find_gateway_for_dm_ms(3001), None);
    }

    #[test]
    fn test_register_overwrites_existing_subscriber() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.affiliate(1001, 91);
        assert!(reg.has_group_members(91));

        reg.register(1001);

        assert!(reg.is_registered(1001));
        reg.deaffiliate(1001, 91);
        assert!(!reg.has_group_members(91));
    }
}

impl Default for StackState {
    fn default() -> Self {
        Self {
            timeslot_alloc: TimeslotAllocator::default(),
            network_connected: false,
            subscribers: SubscriberRegistry::new(),
        }
    }
}
