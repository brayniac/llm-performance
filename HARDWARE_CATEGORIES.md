# Hardware Categories Implementation

## Overview
The system now supports filtering performance data by hardware categories and displays the actual GPU/CPU model used for each benchmark.

## Hardware Categories
- **Consumer GPU**: RTX series (e.g., RTX 4090, RTX 3090), GTX series
- **Consumer CPU**: Ryzen, Intel Core, and other non-datacenter CPUs
- **Datacenter GPU**: A100, H100, L4, L40, V100, T4
- **Datacenter CPU**: Xeon, EPYC processors

## Features

### 1. Hardware Category Filters
- Checkbox filters in the performance grid
- Select one or more categories to filter results
- Empty selection = show all hardware (default)

### 2. GPU Model Display
- Shows actual GPU model next to performance metrics
- Format: "125.3 tok/s (RTX 4090)"
- Removes redundant prefixes like "NVIDIA GeForce"
- Shows "CPU" for CPU-only configurations
- Full hardware details available on hover

### 3. Smart Filtering
- When a hardware category is selected, only results from that category are shown
- The "best quantization" is selected from the filtered results
- Total quantization count remains accurate (counts all, not just filtered)

## Testing

Run this SQL to see hardware categories in your database:
```sql
psql $DATABASE_URL < test_hardware_categories.sql
```

## Implementation Details

### Backend
- `determine_hardware_category()` function in grouped_performance.rs
- Filters applied after counting total quantizations
- Hardware category included in API response

### Frontend  
- HardwareFilters.svelte component for checkboxes
- extractGpuModel() function to parse hardware strings
- GPU model displayed as grey text next to performance metrics
- Column widths adjusted to accommodate GPU model text

### API
- Added `hardware_categories` to GroupedPerformanceRequest
- Added `hardware_category` to QuantizationPerformance response
- Supports multiple category selection via URL parameters