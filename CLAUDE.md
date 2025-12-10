# School Scheduler - Claude Code Guide

## Active Project: FEA (Waco ISD Future Educators Academy)

**Slash commands for FEA work:**
- `/project:fea-init` - First session: extract PDF data, create JSON files
- `/project:fea-work` - Continue work: check progress, tackle next feature

**Data locations:**
- Raw input: `input/fea/*.pdf` + `input/fea/teachers.md`
- Working data: `local-data/fea/*.json`
- Output: `output/fea/`

**FEA constraints:**
- 4 classrooms (Pasisis & Giddings share)
- Multi-grade sections allowed
- Part-time teachers: Pasisis, Giddings
- Edwards replaced Lovelace mid-year (never output "Lovelace")

**Validate JSON before scheduling:**
```bash
python scripts/validate-json.py local-data/fea/
```

---

## Quick Start

```bash
# Build
cargo build --release

# Run demo
cargo run -- demo

# Run with custom data
cargo run -- schedule --data ./local-data --output ./output
```

## Project Structure

- `src/` - Rust source code
  - `types/` - Core data structures
  - `parser/` - JSON loading and validation
  - `scheduler/` - 5-phase scheduling algorithm
  - `validator/` - Constraint checking
  - `reporter/` - Output generation
- `data/demo/` - Demo data (committed)
- `local-data/` - Real data (gitignored)
- `output/` - Generated schedules (gitignored)

## Common Tasks

### Generate Schedule
```bash
cargo run -- schedule --data ./local-data --output ./output
```

### Validate Schedule
```bash
cargo run -- validate --schedule ./output/schedule.json --data ./local-data -v
```

### Individual Reports
```bash
# Student schedule
cargo run -- report --schedule ./output/schedule.json --data ./local-data --student s001

# Teacher schedule
cargo run -- report --schedule ./output/schedule.json --data ./local-data --teacher t001
```

### Run Tests
```bash
cargo test
cargo test -- --nocapture  # Show println output
```

## Algorithm Overview

The scheduler works in 5 phases:

1. **Section Creation** - Creates N sections per course, assigns teachers round-robin
2. **Time Slot Assignment** - CRITICAL: Grade-aware slot assignment to avoid conflicts
3. **Room Assignment** - Assigns rooms based on capacity and required features
4. **ILP Student Assignment** - Uses HiGHS solver to optimize assignments
5. **Post-ILP Optimization** - Balances section enrollments

### Key Formula (ILP)
```
Maximize: Σ(1000 * required_assignment) + Σ((10-rank) * elective_assignment)
Subject to:
  - capacity constraints (hard)
  - time conflict constraints (hard)
  - at most one section per course per student (hard)
```

## Data Formats

### students.json
```json
[
  {
    "id": "s001",
    "name": "Alice Johnson",
    "grade": 10,
    "required_courses": ["math10", "eng10"],
    "elective_preferences": ["art", "music"]
  }
]
```

### teachers.json
```json
[
  {
    "id": "t001",
    "name": "Ms. Anderson",
    "subjects": ["math10", "math11"],
    "max_sections": 4,
    "unavailable": []
  }
]
```

### courses.json
```json
[
  {
    "id": "math10",
    "name": "Algebra 2",
    "max_students": 25,
    "grade_restrictions": [10],
    "required_features": [],
    "sections": 2
  }
]
```

### rooms.json
```json
[
  {
    "id": "101",
    "name": "Room 101",
    "capacity": 30,
    "features": ["lab"],
    "unavailable": []
  }
]
```

## Working with Claude

### Creating Schedules
1. Provide data files or describe your data
2. Claude creates/formats JSON files in `local-data/`
3. Claude runs `cargo run -- schedule`
4. Claude validates and shows results

### Iterating
- "Teacher X has too many back-to-back classes" → modify time constraints
- "Science needs lab rooms" → update room features in courses.json
- "Show Ms. Johnson's schedule" → generate teacher report

### Debugging Tips
- Check `output/schedule.json` for raw assignment data
- Use `cargo run -- validate -v` for detailed constraint analysis
- Look at `output/schedule.md` for human-readable overview

## Performance Targets

| Size | Target |
|------|--------|
| 50 students | <10ms |
| 500 students | <50ms |
| 2000 students | <500ms |
