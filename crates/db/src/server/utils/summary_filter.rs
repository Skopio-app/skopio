use common::{models::Group, time::TimeBucket};

/// A reusable trait for extracting query filters
pub trait SummaryQueryParams {
    fn start(&self) -> Option<i64>;
    fn end(&self) -> Option<i64>;
    fn apps(&self) -> Option<&Vec<String>>;
    fn projects(&self) -> Option<&Vec<String>>;
    fn categories(&self) -> Option<&Vec<String>>;
    fn entities(&self) -> Option<&Vec<String>>;
    fn branches(&self) -> Option<&Vec<String>>;
    fn languages(&self) -> Option<&Vec<String>>;
    fn group_by(&self) -> Option<Group>;
    fn time_bucket(&self) -> Option<TimeBucket>;
}

/// A base struct that holds shared summary query parameters
#[derive(Debug, Clone, Default)]
pub struct SummaryFilters {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub apps: Option<Vec<String>>,
    pub projects: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub entities: Option<Vec<String>>,
    pub branches: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub group_by: Option<Group>,
    pub time_bucket: Option<TimeBucket>,
}

impl SummaryFilters {
    pub fn builder() -> SummaryFiltersBuilder {
        SummaryFiltersBuilder::default()
    }
}

impl SummaryQueryParams for SummaryFilters {
    fn start(&self) -> Option<i64> {
        self.start
    }

    fn end(&self) -> Option<i64> {
        self.end
    }

    fn apps(&self) -> Option<&Vec<String>> {
        self.apps.as_ref()
    }

    fn projects(&self) -> Option<&Vec<String>> {
        self.projects.as_ref()
    }

    fn categories(&self) -> Option<&Vec<String>> {
        self.categories.as_ref()
    }

    fn entities(&self) -> Option<&Vec<String>> {
        self.entities.as_ref()
    }

    fn branches(&self) -> Option<&Vec<String>> {
        self.branches.as_ref()
    }

    fn languages(&self) -> Option<&Vec<String>> {
        self.languages.as_ref()
    }

    fn group_by(&self) -> Option<Group> {
        self.group_by
    }

    fn time_bucket(&self) -> Option<TimeBucket> {
        self.time_bucket
    }
}

/// Builder for Summary Filters
#[derive(Debug, Default)]
pub struct SummaryFiltersBuilder {
    filters: SummaryFilters,
}

impl SummaryFiltersBuilder {
    pub fn start(mut self, value: i64) -> Self {
        self.filters.start = Some(value);
        self
    }

    pub fn end(mut self, value: i64) -> Self {
        self.filters.end = Some(value);
        self
    }

    pub fn apps(mut self, values: Vec<String>) -> Self {
        self.filters.apps = Some(values);
        self
    }

    pub fn projects(mut self, values: Vec<String>) -> Self {
        self.filters.projects = Some(values);
        self
    }

    pub fn entities(mut self, values: Vec<String>) -> Self {
        self.filters.entities = Some(values);
        self
    }

    pub fn branches(mut self, values: Vec<String>) -> Self {
        self.filters.branches = Some(values);
        self
    }

    pub fn categories(mut self, values: Vec<String>) -> Self {
        self.filters.categories = Some(values);
        self
    }

    pub fn languages(mut self, values: Vec<String>) -> Self {
        self.filters.languages = Some(values);
        self
    }

    pub fn group_by(mut self, value: Group) -> Self {
        self.filters.group_by = Some(value);
        self
    }

    pub fn time_bucket(mut self, value: TimeBucket) -> Self {
        self.filters.time_bucket = Some(value);
        self
    }

    pub fn build(self) -> SummaryFilters {
        self.filters
    }
}
