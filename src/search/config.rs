#[derive(Clone)]
pub struct SearchConfig {
    pub max_node_faults: Option<usize>,
    pub max_disk_faults: Option<usize>,
    pub max_msg_drops: Option<usize>,
}

impl SearchConfig {
    pub fn no_faults_with_drops(max_drops: usize) -> Self {
        SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .build()
    }

    pub fn no_faults_no_drops() -> Self {
        SearchConfigBuilder::no_faults().max_msg_drops(0).build()
    }

    pub fn with_node_faults_only(max_node_faults: usize) -> Self {
        SearchConfigBuilder::new()
            .max_disk_faults(0)
            .max_node_faults(max_node_faults)
            .max_msg_drops(0)
            .build()
    }

    pub fn unlimited() -> Self {
        Self {
            max_node_faults: None,
            max_disk_faults: None,
            max_msg_drops: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct SearchConfigBuilder {
    max_node_faults: Option<usize>,
    max_disk_faults: Option<usize>,
    max_msg_drops: Option<usize>,
}

impl SearchConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_node_faults(mut self, max_node_faults: usize) -> Self {
        self.max_node_faults = Some(max_node_faults);
        self
    }

    pub fn max_disk_faults(mut self, max_disk_faults: usize) -> Self {
        self.max_disk_faults = Some(max_disk_faults);
        self
    }

    pub fn max_msg_drops(mut self, max_msg_drops: usize) -> Self {
        self.max_msg_drops = Some(max_msg_drops);
        self
    }

    pub fn no_faults() -> Self {
        Self::new().max_node_faults(0).max_disk_faults(0)
    }

    pub fn no_drops() -> Self {
        Self::new().max_msg_drops(0)
    }

    pub fn build(self) -> SearchConfig {
        SearchConfig {
            max_node_faults: self.max_node_faults,
            max_disk_faults: self.max_disk_faults,
            max_msg_drops: self.max_msg_drops,
        }
    }
}
