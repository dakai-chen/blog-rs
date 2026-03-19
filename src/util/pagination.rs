use std::collections::HashSet;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageNavigation {
    /// 开始页
    pub head: u64,
    /// 上一页
    pub prev: Option<u64>,
    /// 当前页
    pub current_page: u64,
    /// 下一页
    pub next: Option<u64>,
    /// 结束页
    pub tail: Option<u64>,
    /// 页码表
    pub list: Vec<u64>,
}

impl PageNavigation {
    pub fn new<Container>(
        data: &PageData<Container>,
        page: Page,
        nav_len: u64,
        nav_max_page: impl Into<Option<u64>>,
    ) -> Result<Self, NumericalOverflow> {
        let current_page = page.page;
        let nav_max_page = nav_max_page.into();

        // 计算总页数
        let page_n = if page.size == 0 {
            u64::MAX
        } else {
            (data.total.unwrap_or(u64::MAX).saturating_sub(1) / page.size).saturating_add(1)
        };
        let page_n = nav_max_page.map(|m| page_n.min(m)).unwrap_or(page_n).max(1);

        let range = if nav_len == 0 {
            None
        } else {
            // 将当前页限制到合法区域
            let p = current_page.max(1).min(page_n);

            let s; // 导航栏的起始页码
            let e; // 导航栏的结束页码

            if (p - 1) < (page_n - p) {
                s = p.saturating_sub(nav_len / 2).max(1);
                e = s.saturating_sub(1).saturating_add(nav_len).min(page_n);
            } else {
                e = p.saturating_add(nav_len / 2).min(page_n);
                s = e.saturating_sub(nav_len).saturating_add(1).max(1);
            }

            Some((s, e))
        };

        Ok(PageNavigation {
            head: 1,
            prev: if current_page >= 2 {
                Some((current_page - 1).min(page_n))
            } else {
                None
            },
            current_page,
            next: if current_page <= page_n - 1 {
                Some((current_page + 1).max(1))
            } else {
                None
            },
            tail: if nav_max_page.is_none() && (data.total.is_none() || page.size == 0) {
                None
            } else {
                Some(page_n)
            },
            list: range.map(|(s, e)| (s..=e).collect()).unwrap_or_default(),
        })
    }
}

/// 分页结果容器
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageData<Container> {
    /// 当前页数据
    pub items: Container,
    /// 当前页数据条数
    pub count: u64,
    /// 总数据条数
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl<Container> PageData<Container> {
    pub fn new(items: Container, count: u64) -> Self {
        Self {
            items,
            count,
            total: None,
        }
    }

    pub fn with_total(mut self, total: impl Into<Option<u64>>) -> Self {
        self.total = total.into();
        self
    }
}

impl<D> PageData<Vec<D>> {
    pub fn from_vec(items: Vec<D>) -> Result<Self, PageDataCountOverflow<Vec<D>>> {
        let Ok(count) = u64::try_from(items.len()) else {
            return Err(PageDataCountOverflow { items });
        };
        Ok(PageData::new(items, count))
    }
}

impl<D> PageData<HashSet<D>> {
    pub fn from_set(items: HashSet<D>) -> Result<Self, PageDataCountOverflow<HashSet<D>>> {
        let Ok(count) = u64::try_from(items.len()) else {
            return Err(PageDataCountOverflow { items });
        };
        Ok(PageData::new(items, count))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct OptionalPage {
    /// 分页页码
    pub page: Option<u64>,
    /// 分页大小
    pub size: Option<u64>,
}

impl OptionalPage {
    pub fn new(page: impl Into<Option<u64>>, size: impl Into<Option<u64>>) -> Self {
        Self {
            page: page.into(),
            size: size.into(),
        }
    }

    pub fn with_defaults(self, default_page: u64, default_size: u64) -> Page {
        Page::new(
            self.page.unwrap_or(default_page),
            self.size.unwrap_or(default_size),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Page {
    /// 分页页码
    pub page: u64,
    /// 分页大小
    pub size: u64,
}

impl Page {
    pub fn new(page: u64, size: u64) -> Self {
        Self { page, size }
    }

    pub fn to_offset(self) -> Result<Offset, NumericalOverflow> {
        self.page
            .checked_sub(1)
            .and_then(|v| v.checked_mul(self.size))
            .map(|offset| Offset {
                offset,
                size: self.size,
            })
            .ok_or(NumericalOverflow)
    }

    pub fn validate<C1, C2>(self, page_rule: C1, size_rule: C2) -> Result<Self, PageValidateError>
    where
        C1: Contains,
        C2: Contains,
    {
        if !page_rule.contains(self.page) {
            return Err(PageValidateError::InvalidPage {
                page: self,
                rule: page_rule.description(),
            });
        }
        if !size_rule.contains(self.size) {
            return Err(PageValidateError::InvalidSize {
                page: self,
                rule: size_rule.description(),
            });
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Offset {
    /// 偏移量
    pub offset: u64,
    /// 大小
    pub size: u64,
}

impl Offset {
    pub fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }

    pub fn to_range(self) -> Result<Range<u64>, NumericalOverflow> {
        self.offset
            .checked_add(self.size)
            .map(|end| self.offset..end)
            .ok_or(NumericalOverflow)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PageValidateError {
    InvalidPage { page: Page, rule: String },
    InvalidSize { page: Page, rule: String },
}

impl std::fmt::Display for PageValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageValidateError::InvalidPage { page, rule } => {
                write!(f, "分页页码 {} 不合法（要求：{rule}）", page.page)
            }
            PageValidateError::InvalidSize { page, rule } => {
                write!(f, "分页大小 {} 不合法（要求：{rule}）", page.size)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("数值溢出错误")]
pub struct NumericalOverflow;

#[derive(Debug, thiserror::Error)]
#[error("PageData 计数溢出：数据长度超出 u64::MAX 范围")]
pub struct PageDataCountOverflow<Container> {
    pub items: Container,
}

pub trait Contains {
    fn contains(&self, value: u64) -> bool;

    fn description(&self) -> String;
}

impl Contains for Range<u64> {
    fn contains(&self, value: u64) -> bool {
        Range::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for RangeInclusive<u64> {
    fn contains(&self, value: u64) -> bool {
        RangeInclusive::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for RangeFrom<u64> {
    fn contains(&self, value: u64) -> bool {
        RangeFrom::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for RangeTo<u64> {
    fn contains(&self, value: u64) -> bool {
        RangeTo::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for RangeToInclusive<u64> {
    fn contains(&self, value: u64) -> bool {
        RangeToInclusive::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for RangeFull {
    fn contains(&self, _value: u64) -> bool {
        true
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl<const N: usize> Contains for [u64; N] {
    fn contains(&self, value: u64) -> bool {
        self.as_slice().contains(&value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for &[u64] {
    fn contains(&self, value: u64) -> bool {
        <[u64]>::contains(self, &value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}

impl Contains for Vec<u64> {
    fn contains(&self, value: u64) -> bool {
        self.as_slice().contains(&value)
    }

    fn description(&self) -> String {
        format!("{self:?}")
    }
}
