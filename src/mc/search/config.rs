/// Config for the current search iteration.
#[derive(Clone)]
pub struct SearchConfig {
    /// Max number of node faults injected during the search
    pub max_node_faults: Option<usize>,

    /// Max number of node shutdown injected during the search
    pub max_node_shutdown: Option<usize>,

    /// Max number of disk faults injected during the search
    pub max_disk_faults: Option<usize>,

    /// Max number of UDP msg drops injected during the search
    pub max_msg_drops: Option<usize>,
}

impl SearchConfig {
    #[allow(missing_docs)]
    pub fn no_faults_with_drops(max_drops: usize) -> Self {
        SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .build()
    }

    #[allow(missing_docs)]
    pub fn no_faults_no_drops() -> Self {
        SearchConfigBuilder::no_faults()
            .max_msg_drops(0)
            .max_node_shutdown(0)
            .build()
    }

    #[allow(missing_docs)]
    pub fn with_node_faults_only(max_node_faults: usize) -> Self {
        SearchConfigBuilder::new()
            .max_disk_faults(0)
            .max_node_faults(max_node_faults)
            .max_msg_drops(0)
            .max_node_shutdown(0)
            .build()
    }

    #[allow(missing_docs)]
    pub fn with_node_shutdown_only(max_node_shutdown: usize) -> Self {
        SearchConfigBuilder::new()
            .max_disk_faults(0)
            .max_node_faults(0)
            .max_msg_drops(0)
            .max_node_shutdown(max_node_shutdown)
            .build()
    }

    #[allow(missing_docs)]
    pub fn unlimited() -> Self {
        Self {
            max_node_faults: None,
            max_node_shutdown: None,
            max_disk_faults: None,
            max_msg_drops: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents builder for the search config [`SearchConfig`].
#[derive(Default)]
pub struct SearchConfigBuilder {
    max_node_faults: Option<usize>,
    max_node_shutdown: Option<usize>,
    max_disk_faults: Option<usize>,
    max_msg_drops: Option<usize>,
}

impl SearchConfigBuilder {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(missing_docs)]
    pub fn max_node_faults(mut self, max_node_faults: usize) -> Self {
        self.max_node_faults = Some(max_node_faults);
        self
    }

    #[allow(missing_docs)]
    pub fn max_node_shutdown(mut self, max_node_shutdown: usize) -> Self {
        self.max_node_shutdown = Some(max_node_shutdown);
        self
    }

    #[allow(missing_docs)]
    pub fn max_disk_faults(mut self, max_disk_faults: usize) -> Self {
        self.max_disk_faults = Some(max_disk_faults);
        self
    }

    #[allow(missing_docs)]
    pub fn max_msg_drops(mut self, max_msg_drops: usize) -> Self {
        self.max_msg_drops = Some(max_msg_drops);
        self
    }

    #[allow(missing_docs)]
    pub fn no_faults() -> Self {
        Self::new()
            .max_node_faults(0)
            .max_disk_faults(0)
            .max_node_shutdown(0)
    }

    #[allow(missing_docs)]
    pub fn no_drops() -> Self {
        Self::new().max_msg_drops(0)
    }

    #[allow(missing_docs)]
    pub fn build(self) -> SearchConfig {
        SearchConfig {
            max_node_faults: self.max_node_faults,
            max_node_shutdown: self.max_node_shutdown,
            max_disk_faults: self.max_disk_faults,
            max_msg_drops: self.max_msg_drops,
        }
    }
}
