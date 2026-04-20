use anyhow::anyhow;
use systemd::journal::Journal;

use crate::crash::Crash;

type PluginFn = Box<dyn Fn(&mut Journal) -> Option<Crash> + Send + Sync>;

pub struct PluginRegistry {
    filters: Vec<Vec<(&'static str, &'static str)>>,
    plugins: Vec<PluginFn>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn register<R>(
        &mut self,
        filters: &'static [(&'static str, &'static str)],
        detect: fn(&mut Journal) -> Option<R>,
        wrap: fn(R) -> Crash,
    ) -> &mut Self
    where
        R: 'static,
    {
        self.filters.push(filters.to_vec());
        self.plugins.push(Box::new(move |j| detect(j).map(wrap)));
        self
    }

    pub fn install_filters(&self, journal: &mut Journal) -> anyhow::Result<()> {
        let mut first_group = true;

        for group in &self.filters {
            if group.is_empty() {
                continue;
            }

            if !first_group {
                journal
                    .match_or()
                    .map_err(|e| anyhow!("match_or failed: {e}"))?;
            }

            for &(field, value) in group {
                println!("journal fields: {:?}, {:?}", field, value);
                
                journal
                    .match_add(field, value)
                    .map_err(|e| anyhow!("match_add({field}={value}) failed: {e}"))?;
            }

            first_group = false;
        }

        Ok(())
    }

    pub fn run(&self, journal: &mut Journal) -> Option<Crash> {
        self.plugins.iter().find_map(|f| f(journal))
    }
}
