#[derive(Clone)]
pub struct Config {
    pub max_node_faults: Option<usize>,
    pub max_disk_faults: Option<usize>,
    pub max_msg_drops: Option<usize>,
    pub max_depth: Option<usize>,
}

impl Config {
    pub fn no_faults_with_drops() -> Self {
        ConfigBuilder::no_faults().build()
    }

    pub fn no_faults_no_drops() -> Self {
        ConfigBuilder::no_faults().max_msg_drops(0).build()
    }

    pub fn unlimited() -> Self {
        Self {
            max_node_faults: None,
            max_disk_faults: None,
            max_msg_drops: None,
            max_depth: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ConfigBuilder {
    max_node_faults: Option<usize>,
    max_disk_faults: Option<usize>,
    max_msg_drops: Option<usize>,
    max_depth: Option<usize>,
}

impl ConfigBuilder {
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

    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn no_faults() -> Self {
        Self::new().max_node_faults(0).max_disk_faults(0)
    }

    pub fn no_drops() -> Self {
        Self::new().max_msg_drops(0)
    }

    pub fn build(self) -> Config {
        Config {
            max_node_faults: self.max_node_faults,
            max_disk_faults: self.max_disk_faults,
            max_msg_drops: self.max_msg_drops,
            max_depth: self.max_depth,
        }
    }
}
