//! Layout Helpers - Collision detection and spatial utilities
//!
//! Provides helper functions for layout calculations:
//! - Overlap/intersection detection
//! - Finding empty slots
//! - Auto-arrange algorithms
//!
//! # Example
//! ```ignore
//! use egui_cha_ds::molecules::layout_helpers;
//!
//! // Check if two rects overlap
//! let overlap = layout_helpers::overlap_area(rect_a, rect_b);
//!
//! // Find overlapping panes
//! let overlapping = layout_helpers::find_overlapping(my_rect, &other_rects);
//!
//! // Find nearest empty slot for a new pane
//! let slot = layout_helpers::find_nearest_empty_slot(
//!     Vec2::new(200.0, 150.0),
//!     &occupied_rects,
//!     Some(canvas_bounds),
//! );
//! ```

use egui::{Pos2, Rect, Vec2};

/// Gap between panes when auto-arranging (pixels)
pub const DEFAULT_GAP: f32 = 8.0;

/// Calculate the overlap area between two rectangles.
/// Returns 0.0 if they don't overlap.
pub fn overlap_area(a: Rect, b: Rect) -> f32 {
    let x_overlap = (a.max.x.min(b.max.x) - a.min.x.max(b.min.x)).max(0.0);
    let y_overlap = (a.max.y.min(b.max.y) - a.min.y.max(b.min.y)).max(0.0);
    x_overlap * y_overlap
}

/// Check if two rectangles overlap (share any area).
/// Note: touching edges (zero overlap) returns false.
pub fn rects_overlap(a: Rect, b: Rect) -> bool {
    overlap_area(a, b) > 0.0
}

/// Check if two rectangles overlap, with a minimum gap requirement.
/// Returns true if the rects are closer than `min_gap` pixels.
pub fn rects_overlap_with_gap(a: Rect, b: Rect, min_gap: f32) -> bool {
    let a_expanded = a.expand(min_gap / 2.0);
    let b_expanded = b.expand(min_gap / 2.0);
    rects_overlap(a_expanded, b_expanded)
}

/// Find all rectangles that overlap with the given rect.
/// Returns indices of overlapping rects.
pub fn find_overlapping_indices(rect: Rect, others: &[Rect]) -> Vec<usize> {
    others
        .iter()
        .enumerate()
        .filter(|(_, other)| rects_overlap(rect, **other))
        .map(|(i, _)| i)
        .collect()
}

/// Find all rectangles that overlap with the given rect.
/// Returns references to overlapping rects.
pub fn find_overlapping<'a>(rect: Rect, others: &'a [Rect]) -> Vec<&'a Rect> {
    others.iter().filter(|other| rects_overlap(rect, **other)).collect()
}

/// Check if a rectangle overlaps with any in a collection.
pub fn has_any_overlap(rect: Rect, others: &[Rect]) -> bool {
    others.iter().any(|other| rects_overlap(rect, *other))
}

/// Calculate the total overlap area with all other rectangles.
pub fn total_overlap_area(rect: Rect, others: &[Rect]) -> f32 {
    others.iter().map(|other| overlap_area(rect, *other)).sum()
}

/// Find the nearest empty slot for a rectangle of given size.
///
/// Strategy: Start from `preferred_pos` (or origin if None), then search
/// in a spiral pattern outward until an empty slot is found.
///
/// # Arguments
/// * `size` - Size of the rectangle to place
/// * `occupied` - List of already occupied rectangles
/// * `bounds` - Optional canvas bounds (if None, infinite canvas assumed)
/// * `preferred_pos` - Starting position for search (if None, uses origin)
/// * `gap` - Minimum gap between panes
///
/// # Returns
/// Position for the top-left corner of the new rectangle
pub fn find_nearest_empty_slot(
    size: Vec2,
    occupied: &[Rect],
    bounds: Option<Rect>,
    preferred_pos: Option<Pos2>,
    gap: f32,
) -> Pos2 {
    let start = preferred_pos.unwrap_or(Pos2::ZERO);

    // First, try the preferred position
    let candidate = Rect::from_min_size(start, size);
    if !has_any_overlap_with_gap(candidate, occupied, gap) && fits_in_bounds(candidate, bounds) {
        return start;
    }

    // Spiral search pattern
    let step = gap + 20.0; // Search step size
    let max_distance = 2000.0; // Maximum search distance

    let mut distance = step;
    while distance < max_distance {
        // Try positions in a square spiral around the start point
        for angle_step in 0..((distance * 2.0 * std::f32::consts::PI / step) as i32).max(8) {
            let angle = (angle_step as f32) * step / distance;
            let offset = Vec2::new(angle.cos() * distance, angle.sin() * distance);
            let pos = start + offset;

            // Skip negative positions if no bounds specified (assume positive quadrant)
            if bounds.is_none() && (pos.x < 0.0 || pos.y < 0.0) {
                continue;
            }

            let candidate = Rect::from_min_size(pos, size);
            if !has_any_overlap_with_gap(candidate, occupied, gap) && fits_in_bounds(candidate, bounds) {
                return pos;
            }
        }
        distance += step;
    }

    // Fallback: return position far from all others
    let fallback_pos = if let Some(bounds) = bounds {
        Pos2::new(bounds.min.x + gap, bounds.min.y + gap)
    } else {
        Pos2::new(gap, gap)
    };

    fallback_pos
}

/// Find an empty slot using grid-based search (more predictable than spiral).
///
/// Searches positions on a grid, prioritizing positions closer to the preferred position.
pub fn find_empty_slot_grid(
    size: Vec2,
    occupied: &[Rect],
    bounds: Option<Rect>,
    preferred_pos: Option<Pos2>,
    gap: f32,
    grid_size: f32,
) -> Pos2 {
    let start = preferred_pos.unwrap_or_else(|| {
        bounds.map_or(Pos2::ZERO, |b| b.min + Vec2::splat(gap))
    });

    // First, try the preferred position snapped to grid
    let snapped_start = snap_to_grid(start, grid_size);
    let candidate = Rect::from_min_size(snapped_start, size);
    if !has_any_overlap_with_gap(candidate, occupied, gap) && fits_in_bounds(candidate, bounds) {
        return snapped_start;
    }

    // Calculate search bounds
    let search_bounds = bounds.unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::splat(2000.0)));

    // Collect all grid positions and sort by distance from start
    let mut positions: Vec<Pos2> = Vec::new();
    let mut y = search_bounds.min.y + gap;
    while y + size.y <= search_bounds.max.y - gap {
        let mut x = search_bounds.min.x + gap;
        while x + size.x <= search_bounds.max.x - gap {
            positions.push(snap_to_grid(Pos2::new(x, y), grid_size));
            x += grid_size;
        }
        y += grid_size;
    }

    // Sort by distance from preferred position
    positions.sort_by(|a, b| {
        let dist_a = (*a - start).length_sq();
        let dist_b = (*b - start).length_sq();
        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Find first empty slot
    for pos in positions {
        let candidate = Rect::from_min_size(pos, size);
        if !has_any_overlap_with_gap(candidate, occupied, gap) {
            return pos;
        }
    }

    // Fallback
    Pos2::new(search_bounds.min.x + gap, search_bounds.min.y + gap)
}

/// Snap a position to a grid
pub fn snap_to_grid(pos: Pos2, grid_size: f32) -> Pos2 {
    Pos2::new(
        (pos.x / grid_size).round() * grid_size,
        (pos.y / grid_size).round() * grid_size,
    )
}

/// Check if a rectangle fits within optional bounds
fn fits_in_bounds(rect: Rect, bounds: Option<Rect>) -> bool {
    match bounds {
        Some(b) => b.contains_rect(rect),
        None => true,
    }
}

/// Check if a rectangle overlaps with any in a collection, considering gap
fn has_any_overlap_with_gap(rect: Rect, others: &[Rect], gap: f32) -> bool {
    let rect_with_gap = rect.expand(gap / 2.0);
    others.iter().any(|other| {
        let other_with_gap = other.expand(gap / 2.0);
        rects_overlap(rect_with_gap, other_with_gap)
    })
}

/// Calculate bounding box of all rectangles
pub fn bounding_box(rects: &[Rect]) -> Option<Rect> {
    if rects.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for rect in rects {
        min_x = min_x.min(rect.min.x);
        min_y = min_y.min(rect.min.y);
        max_x = max_x.max(rect.max.x);
        max_y = max_y.max(rect.max.y);
    }

    Some(Rect::from_min_max(
        Pos2::new(min_x, min_y),
        Pos2::new(max_x, max_y),
    ))
}

/// Calculate center of mass of rectangles (weighted by area)
pub fn center_of_mass(rects: &[Rect]) -> Option<Pos2> {
    if rects.is_empty() {
        return None;
    }

    let mut total_area = 0.0;
    let mut weighted_x = 0.0;
    let mut weighted_y = 0.0;

    for rect in rects {
        let area = rect.width() * rect.height();
        weighted_x += rect.center().x * area;
        weighted_y += rect.center().y * area;
        total_area += area;
    }

    if total_area > 0.0 {
        Some(Pos2::new(weighted_x / total_area, weighted_y / total_area))
    } else {
        None
    }
}

// ============================================================================
// Overlap Resolution
// ============================================================================

/// Result of overlap resolution
#[derive(Clone, Debug)]
pub struct ResolveResult {
    /// New positions for each rectangle (same order as input)
    pub positions: Vec<Pos2>,
    /// Whether any rectangles were moved
    pub changed: bool,
    /// Number of iterations used
    pub iterations: usize,
}

/// Resolve overlaps between rectangles by pushing them apart.
///
/// Uses an iterative approach where overlapping rectangles are pushed
/// in the direction of minimum overlap until no overlaps remain.
///
/// # Arguments
/// * `rects` - Mutable slice of rectangles to resolve
/// * `gap` - Minimum gap between rectangles
/// * `max_iterations` - Maximum iterations to prevent infinite loops
///
/// # Returns
/// New positions for each rectangle
pub fn resolve_overlaps(
    rects: &[Rect],
    gap: f32,
    max_iterations: usize,
) -> ResolveResult {
    if rects.len() <= 1 {
        return ResolveResult {
            positions: rects.iter().map(|r| r.min).collect(),
            changed: false,
            iterations: 0,
        };
    }

    // Work with positions (top-left corners) and sizes
    let mut positions: Vec<Pos2> = rects.iter().map(|r| r.min).collect();
    let sizes: Vec<Vec2> = rects.iter().map(|r| r.size()).collect();
    let mut changed = false;
    let mut iterations = 0;

    for iter in 0..max_iterations {
        iterations = iter + 1;
        let mut any_moved = false;

        // For each pair of rectangles, check for overlap and push apart
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let rect_i = Rect::from_min_size(positions[i], sizes[i]);
                let rect_j = Rect::from_min_size(positions[j], sizes[j]);

                if rects_overlap_with_gap(rect_i, rect_j, gap) {
                    // Calculate push vector
                    let push = calculate_push_vector(rect_i, rect_j, gap);

                    if push.length_sq() > 0.01 {
                        // Move both rectangles in opposite directions (half each)
                        positions[i] -= push * 0.5;
                        positions[j] += push * 0.5;
                        any_moved = true;
                        changed = true;
                    }
                }
            }
        }

        // Stop if no movement occurred
        if !any_moved {
            break;
        }
    }

    ResolveResult {
        positions,
        changed,
        iterations,
    }
}

/// Calculate the minimum push vector to separate two overlapping rectangles.
///
/// Returns a vector that, when applied to rect_b (and negated for rect_a),
/// will separate the rectangles with the specified gap.
fn calculate_push_vector(a: Rect, b: Rect, gap: f32) -> Vec2 {
    // Calculate overlap in each direction
    let a_center = a.center();
    let b_center = b.center();

    // Direction from a to b
    let direction = b_center - a_center;

    // Calculate required separation
    let combined_half_width = (a.width() + b.width()) / 2.0 + gap;
    let combined_half_height = (a.height() + b.height()) / 2.0 + gap;

    let overlap_x = combined_half_width - direction.x.abs();
    let overlap_y = combined_half_height - direction.y.abs();

    // Choose the direction with minimum overlap (shortest push)
    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        // No overlap
        return Vec2::ZERO;
    }

    if overlap_x < overlap_y {
        // Push horizontally
        let sign = if direction.x >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new(overlap_x * sign, 0.0)
    } else {
        // Push vertically
        let sign = if direction.y >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new(0.0, overlap_y * sign)
    }
}

/// Resolve overlaps while trying to keep rectangles close to their original positions.
///
/// Similar to `resolve_overlaps`, but applies a "spring" force that pulls
/// rectangles back toward their original positions.
pub fn resolve_overlaps_with_anchors(
    rects: &[Rect],
    gap: f32,
    anchor_strength: f32,
    max_iterations: usize,
) -> ResolveResult {
    if rects.len() <= 1 {
        return ResolveResult {
            positions: rects.iter().map(|r| r.min).collect(),
            changed: false,
            iterations: 0,
        };
    }

    let original_positions: Vec<Pos2> = rects.iter().map(|r| r.min).collect();
    let mut positions = original_positions.clone();
    let sizes: Vec<Vec2> = rects.iter().map(|r| r.size()).collect();
    let mut changed = false;
    let mut iterations = 0;

    for iter in 0..max_iterations {
        iterations = iter + 1;
        let mut any_moved = false;

        // Push overlapping rectangles apart
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let rect_i = Rect::from_min_size(positions[i], sizes[i]);
                let rect_j = Rect::from_min_size(positions[j], sizes[j]);

                if rects_overlap_with_gap(rect_i, rect_j, gap) {
                    let push = calculate_push_vector(rect_i, rect_j, gap);

                    if push.length_sq() > 0.01 {
                        positions[i] -= push * 0.5;
                        positions[j] += push * 0.5;
                        any_moved = true;
                        changed = true;
                    }
                }
            }
        }

        // Apply anchor spring force (pull back toward original position)
        for i in 0..positions.len() {
            let anchor_force = (original_positions[i] - positions[i]) * anchor_strength;
            if anchor_force.length_sq() > 0.01 {
                // Only apply if it doesn't cause new overlaps
                let test_pos = positions[i] + anchor_force;
                let test_rect = Rect::from_min_size(test_pos, sizes[i]);

                let would_overlap = (0..positions.len()).any(|j| {
                    if i == j {
                        return false;
                    }
                    let other = Rect::from_min_size(positions[j], sizes[j]);
                    rects_overlap_with_gap(test_rect, other, gap)
                });

                if !would_overlap {
                    positions[i] = test_pos;
                }
            }
        }

        if !any_moved {
            break;
        }
    }

    ResolveResult {
        positions,
        changed,
        iterations,
    }
}

/// Check if a collection of rectangles has any overlaps
pub fn has_overlaps(rects: &[Rect], gap: f32) -> bool {
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            if rects_overlap_with_gap(rects[i], rects[j], gap) {
                return true;
            }
        }
    }
    false
}

/// Count the number of overlapping pairs
pub fn count_overlaps(rects: &[Rect], gap: f32) -> usize {
    let mut count = 0;
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            if rects_overlap_with_gap(rects[i], rects[j], gap) {
                count += 1;
            }
        }
    }
    count
}

// ============================================================================
// Position-based Sorting
// ============================================================================

/// Sort indices by X position (left to right).
/// Used to preserve spatial order when arranging horizontally.
pub fn sort_indices_by_x(rects: &[Rect]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rects.len()).collect();
    indices.sort_by(|&a, &b| {
        rects[a]
            .center()
            .x
            .partial_cmp(&rects[b].center().x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
}

/// Sort indices by Y position (top to bottom).
/// Used to preserve spatial order when arranging vertically.
pub fn sort_indices_by_y(rects: &[Rect]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rects.len()).collect();
    indices.sort_by(|&a, &b| {
        rects[a]
            .center()
            .y
            .partial_cmp(&rects[b].center().y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
}

/// Sort indices in raster scan order (top-to-bottom, left-to-right).
/// Panes on approximately the same row are grouped together.
///
/// # Arguments
/// * `rects` - Rectangles to sort
/// * `row_threshold` - Y distance within which panes are considered same row
pub fn sort_indices_raster(rects: &[Rect], row_threshold: f32) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rects.len()).collect();
    indices.sort_by(|&a, &b| {
        let ya = rects[a].center().y;
        let yb = rects[b].center().y;
        let xa = rects[a].center().x;
        let xb = rects[b].center().x;

        // Group by approximate row first, then by X
        if (ya - yb).abs() > row_threshold {
            ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            xa.partial_cmp(&xb).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    indices
}

/// Sort indices diagonally (top-left to bottom-right).
/// Used for cascade arrangement.
pub fn sort_indices_diagonal(rects: &[Rect]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rects.len()).collect();
    indices.sort_by(|&a, &b| {
        // Sort by sum of x+y (diagonal distance from origin)
        let diag_a = rects[a].center().x + rects[a].center().y;
        let diag_b = rects[b].center().x + rects[b].center().y;
        diag_a
            .partial_cmp(&diag_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
}

// ============================================================================
// Auto-Arrange Algorithms
// ============================================================================

/// Result of auto-arrange operation
#[derive(Clone, Debug)]
pub struct ArrangeResult {
    /// New positions for each rectangle (same order as input)
    pub positions: Vec<Pos2>,
    /// Whether any rectangles were moved
    pub changed: bool,
}

impl From<ResolveResult> for ArrangeResult {
    fn from(result: ResolveResult) -> Self {
        ArrangeResult {
            positions: result.positions,
            changed: result.changed,
        }
    }
}

/// Result of tile arrange operation (includes resizing)
#[derive(Clone, Debug)]
pub struct TileResult {
    /// New positions for each rectangle (same order as input)
    pub positions: Vec<Pos2>,
    /// New sizes for each rectangle (same order as input)
    pub sizes: Vec<Vec2>,
    /// Whether any rectangles were moved or resized
    pub changed: bool,
}

/// Arrange rectangles in a grid layout.
///
/// Places rectangles in a grid with the specified number of columns.
/// All cells have the same size (based on the largest rectangle).
///
/// # Arguments
/// * `rects` - Rectangles to arrange (sizes are preserved)
/// * `columns` - Number of columns (None = auto-calculate)
/// * `origin` - Top-left corner of the grid
/// * `gap` - Gap between rectangles
///
/// # Returns
/// New positions for each rectangle
pub fn arrange_grid(
    rects: &[Rect],
    columns: Option<usize>,
    origin: Pos2,
    gap: f32,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    let count = rects.len();

    // Calculate columns
    let cols = columns.unwrap_or_else(|| {
        match count {
            1 => 1,
            2 => 2,
            3..=4 => 2,
            5..=6 => 3,
            _ => ((count as f32).sqrt().ceil() as usize).max(2),
        }
    });

    // Find max size for uniform grid cells
    let max_width = rects.iter().map(|r| r.width()).fold(0.0f32, f32::max);
    let max_height = rects.iter().map(|r| r.height()).fold(0.0f32, f32::max);

    let cell_width = max_width + gap;
    let cell_height = max_height + gap;

    let mut positions = Vec::with_capacity(count);
    let mut changed = false;

    for (i, rect) in rects.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;

        // Center the rectangle within its cell
        let cell_x = origin.x + (col as f32) * cell_width;
        let cell_y = origin.y + (row as f32) * cell_height;

        // Center smaller rects in their cell
        let offset_x = (max_width - rect.width()) / 2.0;
        let offset_y = (max_height - rect.height()) / 2.0;

        let new_pos = Pos2::new(cell_x + offset_x, cell_y + offset_y);

        if new_pos != rect.min {
            changed = true;
        }
        positions.push(new_pos);
    }

    ArrangeResult { positions, changed }
}

/// Arrange rectangles in a grid with proportional sizing.
///
/// Similar to arrange_grid, but each rectangle keeps its original size
/// and positions are calculated to fit them without overlap.
///
/// Preserves spatial order: panes are sorted in raster scan order
/// (top-to-bottom, left-to-right) before arranging.
pub fn arrange_grid_proportional(
    rects: &[Rect],
    columns: Option<usize>,
    origin: Pos2,
    gap: f32,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    let count = rects.len();
    let cols = columns.unwrap_or_else(|| {
        match count {
            1 => 1,
            2 => 2,
            3..=4 => 2,
            5..=6 => 3,
            _ => ((count as f32).sqrt().ceil() as usize).max(2),
        }
    });

    // Sort by current position in raster order to preserve spatial layout
    // Use average height as row threshold
    let avg_height = rects.iter().map(|r| r.height()).sum::<f32>() / count as f32;
    let row_threshold = avg_height * 0.5;
    let order = sort_indices_raster(rects, row_threshold);

    let rows = (count + cols - 1) / cols;

    // Calculate max width per column and max height per row
    // (based on sorted order)
    let mut col_widths = vec![0.0f32; cols];
    let mut row_heights = vec![0.0f32; rows];

    for (grid_pos, &orig_idx) in order.iter().enumerate() {
        let col = grid_pos % cols;
        let row = grid_pos / cols;
        col_widths[col] = col_widths[col].max(rects[orig_idx].width());
        row_heights[row] = row_heights[row].max(rects[orig_idx].height());
    }

    // Calculate cumulative positions
    let mut col_starts = vec![origin.x];
    for (i, &width) in col_widths.iter().enumerate() {
        col_starts.push(col_starts[i] + width + gap);
    }

    let mut row_starts = vec![origin.y];
    for (i, &height) in row_heights.iter().enumerate() {
        row_starts.push(row_starts[i] + height + gap);
    }

    let mut positions = vec![Pos2::ZERO; count];
    let mut changed = false;

    for (grid_pos, &orig_idx) in order.iter().enumerate() {
        let rect = &rects[orig_idx];
        let col = grid_pos % cols;
        let row = grid_pos / cols;

        // Center the rectangle within its cell
        let cell_width = col_widths[col];
        let cell_height = row_heights[row];
        let offset_x = (cell_width - rect.width()) / 2.0;
        let offset_y = (cell_height - rect.height()) / 2.0;

        let new_pos = Pos2::new(col_starts[col] + offset_x, row_starts[row] + offset_y);

        if new_pos != rect.min {
            changed = true;
        }
        positions[orig_idx] = new_pos;
    }

    ArrangeResult { positions, changed }
}

/// Arrange rectangles in a cascade (diagonal stacking) pattern.
///
/// Each rectangle is offset diagonally from the previous one.
///
/// # Arguments
/// * `rects` - Rectangles to arrange
/// * `origin` - Starting position (top-left of cascade)
/// * `offset` - Diagonal offset between each rectangle
/// * `custom_order` - Optional custom ordering. If None, sorts by diagonal position.
///                    If Some, uses the provided indices (first index = top-left).
pub fn arrange_cascade(
    rects: &[Rect],
    origin: Pos2,
    offset: Vec2,
    custom_order: Option<&[usize]>,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    // Use custom order or sort by diagonal position
    let default_order;
    let order: &[usize] = match custom_order {
        Some(o) => o,
        None => {
            default_order = sort_indices_diagonal(rects);
            &default_order
        }
    };

    let mut positions = vec![Pos2::ZERO; rects.len()];
    let mut changed = false;

    for (cascade_pos, &orig_idx) in order.iter().enumerate() {
        let rect = &rects[orig_idx];
        let new_pos = origin + offset * (cascade_pos as f32);

        if new_pos != rect.min {
            changed = true;
        }
        positions[orig_idx] = new_pos;
    }

    ArrangeResult { positions, changed }
}

/// Arrange rectangles horizontally (side by side).
///
/// Preserves spatial order: panes are sorted by their current X position
/// before arranging, so `A C B` (left to right) stays `A C B`.
pub fn arrange_horizontal(
    rects: &[Rect],
    origin: Pos2,
    gap: f32,
    align_top: bool,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    // Sort by current X position to preserve spatial order
    let order = sort_indices_by_x(rects);

    // Find max height for vertical alignment
    let max_height = if align_top {
        0.0
    } else {
        rects.iter().map(|r| r.height()).fold(0.0f32, f32::max)
    };

    let mut positions = vec![Pos2::ZERO; rects.len()];
    let mut changed = false;
    let mut x = origin.x;

    for &i in &order {
        let rect = &rects[i];
        let y = if align_top {
            origin.y
        } else {
            origin.y + (max_height - rect.height()) / 2.0
        };

        let new_pos = Pos2::new(x, y);

        if new_pos != rect.min {
            changed = true;
        }
        positions[i] = new_pos;

        x += rect.width() + gap;
    }

    ArrangeResult { positions, changed }
}

/// Arrange rectangles vertically (stacked).
///
/// Preserves spatial order: panes are sorted by their current Y position
/// before arranging, so the top-to-bottom order is maintained.
pub fn arrange_vertical(
    rects: &[Rect],
    origin: Pos2,
    gap: f32,
    align_left: bool,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    // Sort by current Y position to preserve spatial order
    let order = sort_indices_by_y(rects);

    // Find max width for horizontal alignment
    let max_width = if align_left {
        0.0
    } else {
        rects.iter().map(|r| r.width()).fold(0.0f32, f32::max)
    };

    let mut positions = vec![Pos2::ZERO; rects.len()];
    let mut changed = false;
    let mut y = origin.y;

    for &i in &order {
        let rect = &rects[i];
        let x = if align_left {
            origin.x
        } else {
            origin.x + (max_width - rect.width()) / 2.0
        };

        let new_pos = Pos2::new(x, y);

        if new_pos != rect.min {
            changed = true;
        }
        positions[i] = new_pos;

        y += rect.height() + gap;
    }

    ArrangeResult { positions, changed }
}

/// Arrange rectangles to fit within bounds, using grid layout.
///
/// Calculates grid size based on available space.
pub fn arrange_fit_bounds(
    rects: &[Rect],
    bounds: Rect,
    gap: f32,
) -> ArrangeResult {
    if rects.is_empty() {
        return ArrangeResult {
            positions: Vec::new(),
            changed: false,
        };
    }

    let count = rects.len();

    // Find average width to estimate columns
    let avg_width: f32 = rects.iter().map(|r| r.width()).sum::<f32>() / count as f32;

    // Calculate how many columns can fit
    let available_width = bounds.width() - gap;
    let cols = ((available_width + gap) / (avg_width + gap)).floor() as usize;
    let cols = cols.max(1).min(count);

    arrange_grid_proportional(
        rects,
        Some(cols),
        bounds.min + Vec2::splat(gap / 2.0),
        gap,
    )
}

/// Arrange rectangles in a tile layout filling available bounds (macOS-style).
///
/// This function resizes panes to fill the available space evenly.
/// Panes are sorted by their current position (raster order) before tiling.
///
/// # Layout patterns by pane count:
/// - 1: Full screen
/// - 2: Left/Right split (50/50)
/// - 3: Left (50%) + Right stack (2 x 25%)
/// - 4: 2x2 grid
/// - 5: Left stack (2) + Right stack (3)
/// - 6+: Dynamic grid
///
/// # Arguments
/// * `rects` - Current rectangles (positions used for sorting)
/// * `bounds` - Available area to fill
/// * `gap` - Gap between tiles
/// * `min_sizes` - Optional minimum sizes for each pane
pub fn arrange_tile(
    rects: &[Rect],
    bounds: Rect,
    gap: f32,
    min_sizes: Option<&[Vec2]>,
) -> TileResult {
    let count = rects.len();

    if count == 0 {
        return TileResult {
            positions: Vec::new(),
            sizes: Vec::new(),
            changed: false,
        };
    }

    // Sort by raster order to preserve spatial layout
    let avg_height = rects.iter().map(|r| r.height()).sum::<f32>() / count as f32;
    let order = sort_indices_raster(rects, avg_height * 0.5);

    let mut positions = vec![Pos2::ZERO; count];
    let mut sizes = vec![Vec2::ZERO; count];
    let mut changed = false;

    let available_width = bounds.width() - gap;
    let available_height = bounds.height() - gap;

    match count {
        1 => {
            // Full screen
            let orig_idx = order[0];
            let new_pos = bounds.min + Vec2::splat(gap / 2.0);
            let new_size = Vec2::new(available_width, available_height);
            let new_size = apply_min_size(new_size, min_sizes, orig_idx);

            positions[orig_idx] = new_pos;
            sizes[orig_idx] = new_size;
            changed = check_changed(&rects[orig_idx], new_pos, new_size);
        }
        2 => {
            // Left/Right split
            let half_width = (available_width - gap) / 2.0;

            for (tile_idx, &orig_idx) in order.iter().enumerate() {
                let x = bounds.min.x + gap / 2.0 + (tile_idx as f32) * (half_width + gap);
                let new_pos = Pos2::new(x, bounds.min.y + gap / 2.0);
                let new_size = Vec2::new(half_width, available_height);
                let new_size = apply_min_size(new_size, min_sizes, orig_idx);

                positions[orig_idx] = new_pos;
                sizes[orig_idx] = new_size;
                if check_changed(&rects[orig_idx], new_pos, new_size) {
                    changed = true;
                }
            }
        }
        3 => {
            // Left (50%) + Right stack (2 x 50%)
            let left_width = (available_width - gap) / 2.0;
            let right_width = left_width;
            let right_height = (available_height - gap) / 2.0;

            // First pane: left side
            let orig_idx = order[0];
            let new_pos = Pos2::new(bounds.min.x + gap / 2.0, bounds.min.y + gap / 2.0);
            let new_size = Vec2::new(left_width, available_height);
            let new_size = apply_min_size(new_size, min_sizes, orig_idx);
            positions[orig_idx] = new_pos;
            sizes[orig_idx] = new_size;
            if check_changed(&rects[orig_idx], new_pos, new_size) {
                changed = true;
            }

            // Second pane: right top
            let orig_idx = order[1];
            let new_pos = Pos2::new(
                bounds.min.x + gap / 2.0 + left_width + gap,
                bounds.min.y + gap / 2.0,
            );
            let new_size = Vec2::new(right_width, right_height);
            let new_size = apply_min_size(new_size, min_sizes, orig_idx);
            positions[orig_idx] = new_pos;
            sizes[orig_idx] = new_size;
            if check_changed(&rects[orig_idx], new_pos, new_size) {
                changed = true;
            }

            // Third pane: right bottom
            let orig_idx = order[2];
            let new_pos = Pos2::new(
                bounds.min.x + gap / 2.0 + left_width + gap,
                bounds.min.y + gap / 2.0 + right_height + gap,
            );
            let new_size = Vec2::new(right_width, right_height);
            let new_size = apply_min_size(new_size, min_sizes, orig_idx);
            positions[orig_idx] = new_pos;
            sizes[orig_idx] = new_size;
            if check_changed(&rects[orig_idx], new_pos, new_size) {
                changed = true;
            }
        }
        4 => {
            // 2x2 grid
            let cell_width = (available_width - gap) / 2.0;
            let cell_height = (available_height - gap) / 2.0;

            for (tile_idx, &orig_idx) in order.iter().enumerate() {
                let col = tile_idx % 2;
                let row = tile_idx / 2;
                let x = bounds.min.x + gap / 2.0 + (col as f32) * (cell_width + gap);
                let y = bounds.min.y + gap / 2.0 + (row as f32) * (cell_height + gap);
                let new_pos = Pos2::new(x, y);
                let new_size = Vec2::new(cell_width, cell_height);
                let new_size = apply_min_size(new_size, min_sizes, orig_idx);

                positions[orig_idx] = new_pos;
                sizes[orig_idx] = new_size;
                if check_changed(&rects[orig_idx], new_pos, new_size) {
                    changed = true;
                }
            }
        }
        _ => {
            // Dynamic grid for 5+ panes
            let (cols, rows) = calculate_grid_dimensions(count);
            let cell_width = (available_width - gap * (cols as f32 - 1.0)) / cols as f32;
            let cell_height = (available_height - gap * (rows as f32 - 1.0)) / rows as f32;

            for (tile_idx, &orig_idx) in order.iter().enumerate() {
                let col = tile_idx % cols;
                let row = tile_idx / cols;
                let x = bounds.min.x + gap / 2.0 + (col as f32) * (cell_width + gap);
                let y = bounds.min.y + gap / 2.0 + (row as f32) * (cell_height + gap);
                let new_pos = Pos2::new(x, y);
                let new_size = Vec2::new(cell_width, cell_height);
                let new_size = apply_min_size(new_size, min_sizes, orig_idx);

                positions[orig_idx] = new_pos;
                sizes[orig_idx] = new_size;
                if check_changed(&rects[orig_idx], new_pos, new_size) {
                    changed = true;
                }
            }
        }
    }

    TileResult {
        positions,
        sizes,
        changed,
    }
}

/// Calculate optimal grid dimensions for a given count
fn calculate_grid_dimensions(count: usize) -> (usize, usize) {
    match count {
        0 => (0, 0),
        1 => (1, 1),
        2 => (2, 1),
        3 => (3, 1),
        4 => (2, 2),
        5 => (3, 2),
        6 => (3, 2),
        7..=9 => (3, 3),
        10..=12 => (4, 3),
        _ => {
            let cols = (count as f32).sqrt().ceil() as usize;
            let rows = (count + cols - 1) / cols;
            (cols, rows)
        }
    }
}

/// Apply minimum size constraint
fn apply_min_size(size: Vec2, min_sizes: Option<&[Vec2]>, idx: usize) -> Vec2 {
    match min_sizes {
        Some(mins) if idx < mins.len() => Vec2::new(
            size.x.max(mins[idx].x),
            size.y.max(mins[idx].y),
        ),
        _ => size,
    }
}

/// Check if position or size changed
fn check_changed(rect: &Rect, new_pos: Pos2, new_size: Vec2) -> bool {
    rect.min != new_pos || rect.size() != new_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_area() {
        // No overlap
        let a = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Rect::from_min_size(Pos2::new(20.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(overlap_area(a, b), 0.0);

        // Full overlap (same rect)
        assert_eq!(overlap_area(a, a), 100.0);

        // Partial overlap
        let c = Rect::from_min_size(Pos2::new(5.0, 5.0), Vec2::new(10.0, 10.0));
        assert_eq!(overlap_area(a, c), 25.0); // 5x5 overlap
    }

    #[test]
    fn test_rects_overlap() {
        let a = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Rect::from_min_size(Pos2::new(20.0, 0.0), Vec2::new(10.0, 10.0));
        let c = Rect::from_min_size(Pos2::new(5.0, 5.0), Vec2::new(10.0, 10.0));

        assert!(!rects_overlap(a, b));
        assert!(rects_overlap(a, c));
    }

    #[test]
    fn test_find_overlapping() {
        let target = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let others = vec![
            Rect::from_min_size(Pos2::new(5.0, 5.0), Vec2::new(10.0, 10.0)),   // overlaps
            Rect::from_min_size(Pos2::new(20.0, 0.0), Vec2::new(10.0, 10.0)),  // no overlap
            Rect::from_min_size(Pos2::new(0.0, 5.0), Vec2::new(5.0, 10.0)),    // overlaps
        ];

        let overlapping = find_overlapping(target, &others);
        assert_eq!(overlapping.len(), 2);
    }

    #[test]
    fn test_find_nearest_empty_slot() {
        let size = Vec2::new(100.0, 100.0);
        let occupied = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
        ];

        let slot = find_nearest_empty_slot(size, &occupied, None, Some(Pos2::ZERO), DEFAULT_GAP);

        // Should not overlap with occupied rect
        let new_rect = Rect::from_min_size(slot, size);
        assert!(!has_any_overlap(new_rect, &occupied));
    }

    #[test]
    fn test_bounding_box() {
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(10.0, 10.0)),
        ];

        let bbox = bounding_box(&rects).unwrap();
        assert_eq!(bbox.min, Pos2::new(0.0, 0.0));
        assert_eq!(bbox.max, Pos2::new(30.0, 30.0));
    }

    #[test]
    fn test_has_overlaps() {
        // No overlaps
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            Rect::from_min_size(Pos2::new(20.0, 0.0), Vec2::new(10.0, 10.0)),
        ];
        assert!(!has_overlaps(&rects, 0.0));

        // With overlaps
        let rects_overlapping = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            Rect::from_min_size(Pos2::new(5.0, 5.0), Vec2::new(10.0, 10.0)),
        ];
        assert!(has_overlaps(&rects_overlapping, 0.0));
    }

    #[test]
    fn test_resolve_overlaps_no_overlap() {
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            Rect::from_min_size(Pos2::new(20.0, 0.0), Vec2::new(10.0, 10.0)),
        ];

        let result = resolve_overlaps(&rects, DEFAULT_GAP, 100);
        assert!(!result.changed);
    }

    #[test]
    fn test_resolve_overlaps_with_overlap() {
        // Two overlapping rectangles
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(100.0, 100.0)),
        ];

        let result = resolve_overlaps(&rects, DEFAULT_GAP, 100);
        assert!(result.changed);

        // Verify no overlaps in result
        let resolved_rects: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();

        assert!(!has_overlaps(&resolved_rects, DEFAULT_GAP));
    }

    #[test]
    fn test_resolve_overlaps_multiple() {
        // Three overlapping rectangles in a cluster
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(50.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(25.0, 50.0), Vec2::new(100.0, 100.0)),
        ];

        let result = resolve_overlaps(&rects, DEFAULT_GAP, 100);
        assert!(result.changed);

        // Verify no overlaps in result
        let resolved_rects: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();

        assert!(!has_overlaps(&resolved_rects, DEFAULT_GAP));
    }

    #[test]
    fn test_count_overlaps() {
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(50.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(200.0, 0.0), Vec2::new(100.0, 100.0)), // no overlap
        ];

        assert_eq!(count_overlaps(&rects, 0.0), 1);
    }

    #[test]
    fn test_arrange_grid() {
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
        ];

        let result = arrange_grid(&rects, Some(2), Pos2::ZERO, DEFAULT_GAP);
        assert!(result.changed);
        assert_eq!(result.positions.len(), 4);

        // Check grid layout (2x2)
        // First row
        assert_eq!(result.positions[0].x, 0.0);
        assert_eq!(result.positions[1].x, 100.0 + DEFAULT_GAP);
        // Second row
        assert_eq!(result.positions[2].y, 100.0 + DEFAULT_GAP);
        assert_eq!(result.positions[3].y, 100.0 + DEFAULT_GAP);

        // No overlaps in result
        let arranged: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();
        assert!(!has_overlaps(&arranged, DEFAULT_GAP));
    }

    #[test]
    fn test_arrange_grid_proportional() {
        // Different sized rectangles
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 50.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(80.0, 100.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(120.0, 80.0)),
        ];

        let result = arrange_grid_proportional(&rects, Some(2), Pos2::ZERO, DEFAULT_GAP);
        assert!(result.changed);

        // No overlaps
        let arranged: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();
        assert!(!has_overlaps(&arranged, DEFAULT_GAP));
    }

    #[test]
    fn test_arrange_horizontal() {
        // Rects with different X positions - should preserve left-to-right order
        // Order by X: rect[0] at x=50, rect[1] at x=200, rect[2] at x=100
        // So sorted order is: 0, 2, 1
        let rects = vec![
            Rect::from_min_size(Pos2::new(50.0, 0.0), Vec2::new(100.0, 50.0)),   // leftmost
            Rect::from_min_size(Pos2::new(200.0, 0.0), Vec2::new(80.0, 100.0)),  // rightmost
            Rect::from_min_size(Pos2::new(100.0, 0.0), Vec2::new(60.0, 70.0)),   // middle
        ];

        let result = arrange_horizontal(&rects, Pos2::new(10.0, 10.0), DEFAULT_GAP, false);
        assert!(result.changed);

        // Positions should preserve spatial order (left to right): 0 -> 2 -> 1
        // rect[0] is leftmost, so it should be at x=10.0
        assert_eq!(result.positions[0].x, 10.0);
        // rect[2] is middle, so it should be at x=10+100+gap = 118.0
        assert_eq!(result.positions[2].x, 10.0 + 100.0 + DEFAULT_GAP);
        // rect[1] is rightmost, so it should be at x=10+100+gap+60+gap = 186.0
        assert_eq!(result.positions[1].x, 10.0 + 100.0 + DEFAULT_GAP + 60.0 + DEFAULT_GAP);

        // No overlaps
        let arranged: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();
        assert!(!has_overlaps(&arranged, DEFAULT_GAP));
    }

    #[test]
    fn test_arrange_vertical() {
        // Rects with different Y positions - should preserve top-to-bottom order
        // Order by Y: rect[0] at y=20, rect[1] at y=200, rect[2] at y=80
        // So sorted order is: 0, 2, 1
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 20.0), Vec2::new(100.0, 50.0)),   // topmost
            Rect::from_min_size(Pos2::new(0.0, 200.0), Vec2::new(80.0, 100.0)),  // bottommost
            Rect::from_min_size(Pos2::new(0.0, 80.0), Vec2::new(60.0, 70.0)),    // middle
        ];

        let result = arrange_vertical(&rects, Pos2::new(10.0, 10.0), DEFAULT_GAP, false);
        assert!(result.changed);

        // Positions should preserve spatial order (top to bottom): 0 -> 2 -> 1
        // rect[0] is topmost, so it should be at y=10.0
        assert_eq!(result.positions[0].y, 10.0);
        // rect[2] is middle, so it should be at y=10+50+gap = 68.0
        assert_eq!(result.positions[2].y, 10.0 + 50.0 + DEFAULT_GAP);
        // rect[1] is bottommost, so it should be at y=10+50+gap+70+gap = 146.0
        assert_eq!(result.positions[1].y, 10.0 + 50.0 + DEFAULT_GAP + 70.0 + DEFAULT_GAP);

        // No overlaps
        let arranged: Vec<Rect> = result
            .positions
            .iter()
            .zip(rects.iter())
            .map(|(pos, r)| Rect::from_min_size(*pos, r.size()))
            .collect();
        assert!(!has_overlaps(&arranged, DEFAULT_GAP));
    }

    #[test]
    fn test_arrange_cascade() {
        let rects = vec![
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 150.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 150.0)),
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 150.0)),
        ];

        let offset = Vec2::new(30.0, 30.0);
        // Use custom order: 0 -> 1 -> 2
        let order = vec![0, 1, 2];
        let result = arrange_cascade(&rects, Pos2::new(10.0, 10.0), offset, Some(&order));
        assert!(result.changed);

        // Diagonal offset in specified order
        assert_eq!(result.positions[0], Pos2::new(10.0, 10.0));
        assert_eq!(result.positions[1], Pos2::new(40.0, 40.0));
        assert_eq!(result.positions[2], Pos2::new(70.0, 70.0));
    }

    #[test]
    fn test_arrange_cascade_custom_order() {
        // Test with custom order (e.g., Z-order based)
        let rects = vec![
            Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(200.0, 150.0)),
            Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(200.0, 150.0)),
            Rect::from_min_size(Pos2::new(150.0, 150.0), Vec2::new(200.0, 150.0)),
        ];

        let offset = Vec2::new(30.0, 30.0);
        // Custom order: 2 (back) -> 0 (middle) -> 1 (front)
        let order = vec![2, 0, 1];
        let result = arrange_cascade(&rects, Pos2::new(10.0, 10.0), offset, Some(&order));
        assert!(result.changed);

        // rect[2] is first (top-left)
        assert_eq!(result.positions[2], Pos2::new(10.0, 10.0));
        // rect[0] is second
        assert_eq!(result.positions[0], Pos2::new(40.0, 40.0));
        // rect[1] is third (bottom-right)
        assert_eq!(result.positions[1], Pos2::new(70.0, 70.0));
    }
}
