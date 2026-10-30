use std::cmp::Ordering;

use super::usage::{AppRow, ProcessRow, Usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Name,
    Cpu,
    Memory,
    Gpu,
    Vram,
    Disk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: Column,
    pub order: Order,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            column: Column::Cpu,
            order: Order::Descending,
        }
    }
}

impl Sort {
    pub fn by(self, column: Column) -> Self {
        if self.column != column {
            return Self {
                column,
                order: column.natural(),
            };
        }
        let order = match self.order {
            Order::Ascending => Order::Descending,
            Order::Descending => Order::Ascending,
        };
        Self { column, order }
    }
}

impl Column {
    fn natural(self) -> Order {
        match self {
            Column::Name => Order::Ascending,
            _ => Order::Descending,
        }
    }
}

pub trait Row {
    fn name(&self) -> &str;
    fn usage(&self) -> &Usage;
}

impl Row for ProcessRow {
    fn name(&self) -> &str {
        &self.name
    }

    fn usage(&self) -> &Usage {
        &self.usage
    }
}

impl Row for AppRow {
    fn name(&self) -> &str {
        &self.name
    }

    fn usage(&self) -> &Usage {
        &self.usage
    }
}

pub fn sort<R: Row>(rows: &mut [R], sort: Sort) {
    rows.sort_by(|left, right| {
        let ordering = compare(left, right, sort.column);
        let ordering = match sort.order {
            Order::Ascending => ordering,
            Order::Descending => ordering.reverse(),
        };
        ordering.then_with(|| compare(left, right, Column::Name))
    });
}

fn compare<R: Row>(left: &R, right: &R, column: Column) -> Ordering {
    let (a, b) = (left.usage(), right.usage());
    match column {
        Column::Name => left.name().to_lowercase().cmp(&right.name().to_lowercase()),
        Column::Cpu => a.cpu.total_cmp(&b.cpu),
        Column::Memory => a.memory.cmp(&b.memory),
        Column::Gpu => a.gpu.unwrap_or(-1.0).total_cmp(&b.gpu.unwrap_or(-1.0)),
        Column::Vram => a.vram.cmp(&b.vram),
        Column::Disk => a.disk.total_cmp(&b.disk),
    }
}

pub fn top<R: Row>(rows: &[R], column: Column) -> Option<&R> {
    rows.iter()
        .max_by(|left, right| compare(*left, *right, column))
}
