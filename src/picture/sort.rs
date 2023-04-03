use super::PictureData;

#[derive(Debug, Default, Clone)]
pub enum SortField {
    #[default]
    CaptureDate,
}

#[derive(Debug, Default, Clone)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default, Clone)]
pub struct SortSettings {
    sort_field: SortField,
    sort_direction: SortDirection,
}

impl SortSettings {
    pub fn sort(&self, left: &PictureData, right: &PictureData) -> std::cmp::Ordering {
        let cmp = match self.sort_field {
            SortField::CaptureDate => match (left.capture_time, right.capture_time) {
                (Some(l), Some(r)) => l.cmp(&r),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        };

        match self.sort_direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    }
}
