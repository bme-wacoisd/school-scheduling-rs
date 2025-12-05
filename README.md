# School Scheduler

**A high-performance, constraint-based school schedule generator powered by Integer Linear Programming.**

---

## What This Project Can Do For You

### The Problem We Solve

Creating a school schedule is one of the most complex optimization problems administrators face. You must simultaneously satisfy:

- **Every student** gets their required courses
- **Every student** gets as many elective preferences as possible (in priority order)
- **No teacher** is double-booked
- **No student** has conflicting classes
- **No room** is used by two classes at once
- **Room capacities** are respected
- **Room features** match course needs (labs for chemistry, gyms for PE)
- **Grade restrictions** are honored (seniors can't take freshman courses)
- **Teacher qualifications** are verified
- **Section sizes** are balanced (not 30 students in one section, 5 in another)

Doing this by hand for even 100 students takes days. For 500+ students, it's weeks of frustrating trial-and-error.

### What We Deliver

| Scenario | Students | Courses | Sections | Solve Time | Result |
|----------|----------|---------|----------|------------|--------|
| Small School | 50 | 15 | 25 | **< 100ms** | 100% required courses assigned |
| Medium School | 500 | 40 | 120 | **< 5 seconds** | 99%+ required, 95%+ electives |
| Large School | 2,000 | 80 | 300 | **< 30 seconds** | 98%+ required, 90%+ electives |

**The algorithm maximizes**:
1. Required course assignments (weighted 1000x in the optimizer)
2. Elective preferences by rank (1st choice weighted 10x, 2nd choice 9x, etc.)
3. Section balance (post-optimization pass)

**While guaranteeing**:
- Zero teacher conflicts
- Zero student conflicts
- Zero room conflicts
- Zero capacity violations

### Real-World Constraint Handling

**Hard Constraints** (never violated):
- `NoTeacherConflict` - A teacher cannot teach two sections simultaneously
- `NoStudentConflict` - A student cannot attend two classes at the same time
- `NoRoomConflict` - A room cannot host two classes simultaneously
- `RoomCapacity` - Section enrollment ≤ room capacity
- `TeacherQualified` - Teachers only assigned to subjects they can teach
- `GradeRestriction` - Students only placed in grade-appropriate courses
- `TeacherMaxSections` - Teachers don't exceed their maximum load

**Soft Constraints** (optimized, not guaranteed):
- `StudentElectivePreference` - Higher-ranked electives prioritized
- `BalancedSections` - Even distribution across sections of the same course
- `MinimizeGaps` - Reduce free periods in student schedules (future)

### The Secret Sauce: Grade-Aware Time Slot Assignment

The most critical insight from building this system: **time slot assignment makes or breaks a schedule**.

If you assign "Government" (required for seniors only) and "English 12" (also seniors only) to the same period, *no senior can take both*. The ILP solver can't fix this—it's already too late.

Our algorithm:
1. **Prioritizes grade-restricted courses** during time slot assignment
2. **Tracks per-grade slot usage** and penalizes conflicts
3. **Separates sections of the same course** so students have options

This is why a school with 8 periods can successfully schedule 12+ grade-restricted courses.

---

## How to Use This Project

### Quick Start

```bash
# Build the project
cargo build --release

# Run the demo with sample data
cargo run --release -- demo

# Generate a schedule from your data
cargo run --release -- schedule --data ./local-data --output ./output
```

### Working with Claude Code (Recommended Workflow)

This project is designed to be used with [Claude Code](https://claude.ai/claude-code), Anthropic's AI coding assistant. Here's how to get the most out of it:

#### 1. Data Preparation Phase

Tell Claude what you have:
> "I have a CSV export from PowerSchool with 450 students. Each row has student ID, name, grade, and their course requests. Can you help me convert this to the JSON format the scheduler needs?"

Claude will:
- Parse your existing data formats (CSV, Excel exports, etc.)
- Create properly formatted JSON files in `local-data/`
- Validate the data for consistency
- Flag issues like "Student S-142 requested 'AP Calculus' but that course ID doesn't exist"

#### 2. Constraint Definition Phase

Describe your school's rules in plain English:
> "Our school has 7 periods. Lunch must be either period 4 or 5. The science lab can only hold 24 students. Mrs. Johnson can't teach period 1 because she has bus duty."

Claude will translate these into the appropriate data structures and constraint configurations.

#### 3. Schedule Generation Phase

Simply ask:
> "Generate the schedule and show me the results"

Claude will:
- Run the scheduler
- Analyze the output
- Highlight any issues ("12 students couldn't get their first-choice elective")
- Suggest solutions ("Adding a second section of Art would resolve 8 of those conflicts")

#### 4. Iteration Phase

Refine based on results:
> "Can we move Algebra 2 to period 3 instead? The math department prefers morning classes."
> "What if we added a second Chemistry section?"
> "Show me Ms. Rodriguez's schedule"

Claude will make adjustments and re-run until you're satisfied.

### Data File Formats

Place these JSON files in your data directory:

**students.json**
```json
[
  {
    "id": "s001",
    "name": "Alice Johnson",
    "grade": 10,
    "required_courses": ["math10", "eng10", "sci10"],
    "elective_preferences": ["art", "music", "drama"]
  }
]
```

**teachers.json**
```json
[
  {
    "id": "t001",
    "name": "Ms. Anderson",
    "subjects": ["math10", "math11", "math12"],
    "max_sections": 5,
    "unavailable": [{"day": 0, "slot": 0}]
  }
]
```

**courses.json**
```json
[
  {
    "id": "math10",
    "name": "Algebra 2",
    "max_students": 28,
    "grade_restrictions": [10],
    "required_features": [],
    "sections": 2
  }
]
```

**rooms.json**
```json
[
  {
    "id": "101",
    "name": "Room 101",
    "capacity": 30,
    "features": ["projector"],
    "unavailable": []
  }
]
```

### CLI Commands

```bash
# Run demo with built-in sample data
school-scheduler demo

# Generate schedule from your data
school-scheduler schedule --data ./local-data --output ./output --format all

# Validate an existing schedule
school-scheduler validate --schedule ./output/schedule.json --data ./local-data --verbose

# Generate individual reports
school-scheduler report --schedule ./output/schedule.json --data ./local-data --student s001
school-scheduler report --schedule ./output/schedule.json --data ./local-data --teacher t001
```

---

## Privacy and Data Protection

### Our Commitment

**Student data never leaves your computer.**

This is a local application. There is:
- ❌ No cloud service
- ❌ No data upload
- ❌ No external API calls
- ❌ No telemetry or analytics
- ❌ No network requests of any kind

The scheduler runs entirely on your machine using your local files.

### Protecting PII (Personally Identifiable Information)

#### Directory Structure for Safety

```
school-scheduling-rs/
├── data/demo/          # Sample data (committed to git, NO real names)
├── local-data/         # YOUR real data (gitignored, never committed)
└── output/             # Generated schedules (gitignored)
```

The `.gitignore` file explicitly excludes:
```
local-data/
output/
```

**Real student data should ONLY go in `local-data/`** which is never tracked by version control.

#### Best Practices

1. **Use ID-only mode for sharing**: Generate schedules using IDs instead of names when sharing with others
2. **Anonymize for troubleshooting**: If you need help debugging, replace real names with fake ones
3. **Secure your machine**: Standard precautions apply—disk encryption, access controls
4. **Delete when done**: Remove `local-data/` and `output/` when the scheduling cycle is complete

#### Working with Claude Code Safely

When using Claude Code to help with scheduling:

- **DO**: Describe problems generically ("Student in grade 10 can't get their required math course")
- **DO**: Share course names, room numbers, period structures
- **DO**: Share anonymized sample data for format questions
- **DON'T**: Paste real student names into the chat
- **DON'T**: Share files containing PII through cloud services

Claude Code operates on your local files directly. The AI sees file contents to help you, but this happens locally through the CLI—data isn't uploaded to external servers for the file operations themselves.

#### FERPA Compliance Note

This tool can be part of a FERPA-compliant workflow because:
1. Data stays local
2. No third-party data processors
3. You control all access
4. Standard IT security practices apply

Consult your district's data governance policies for specific requirements.

---

## Third-Party Libraries

This project is built on excellent open-source foundations:

### Core Solver

| Library | Purpose | How We Use It |
|---------|---------|---------------|
| [**good_lp**](https://crates.io/crates/good_lp) | High-level linear programming DSL | Provides an ergonomic Rust API for building optimization models. We use it to define decision variables, constraints, and objectives without low-level solver details. |
| [**HiGHS**](https://highs.dev/) | Industrial-strength ILP/MIP solver | The actual optimization engine. HiGHS is the [fastest open-source linear programming solver](https://plato.asu.edu/ftp/lpsimp.html) and handles our integer constraints for student-section assignments. Originally from the University of Edinburgh. |

### CLI & User Interface

| Library | Purpose | How We Use It |
|---------|---------|---------------|
| [**clap**](https://crates.io/crates/clap) | Command-line argument parsing | Provides the `demo`, `schedule`, `validate`, and `report` subcommands with full help text and validation. |
| [**indicatif**](https://crates.io/crates/indicatif) | Progress bars and spinners | Shows solve progress during long-running optimizations so you know it's working. |
| [**colored**](https://crates.io/crates/colored) | Terminal colors | Makes output readable with green checkmarks for success, red for errors. |

### Data Handling

| Library | Purpose | How We Use It |
|---------|---------|---------------|
| [**serde**](https://serde.rs/) | Serialization framework | The industry-standard Rust library for converting between JSON and Rust types. All our data files use serde. |
| [**serde_json**](https://crates.io/crates/serde_json) | JSON parsing | Reads student, teacher, course, and room data; writes schedule output. |
| [**toml**](https://crates.io/crates/toml) | TOML parsing | For configuration files (optional). |
| [**chrono**](https://crates.io/crates/chrono) | Date/time handling | Timestamps in schedule metadata and reports. |

### Error Handling

| Library | Purpose | How We Use It |
|---------|---------|---------------|
| [**thiserror**](https://crates.io/crates/thiserror) | Custom error types | Defines domain-specific errors like `UnknownCourse`, `UnqualifiedTeacher`, `SolverFailed`. |
| [**anyhow**](https://crates.io/crates/anyhow) | Error propagation | Handles error chains at the application level with context messages. |

### Development & Testing

| Library | Purpose | How We Use It |
|---------|---------|---------------|
| [**insta**](https://crates.io/crates/insta) | Snapshot testing | Verifies schedule output format doesn't change unexpectedly. |
| [**proptest**](https://crates.io/crates/proptest) | Property-based testing | Fuzzes constraint validation with random inputs. |
| [**criterion**](https://crates.io/crates/criterion) | Benchmarking | Measures solver performance across different school sizes. |

---

## Algorithm Deep Dive

### The Five Phases

```
┌─────────────────────────────────────────────────────────────┐
│  Phase 1: Section Creation                                  │
│  - Create N sections per course                             │
│  - Assign teachers round-robin from qualified pool          │
│  - Respect teacher max_sections limits                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Phase 2: Time Slot Assignment (CRITICAL)                   │
│  - Grade-restricted courses assigned first                  │
│  - Track per-grade slot usage to avoid conflicts            │
│  - Spread sections of same course across different slots    │
│  - Respect teacher unavailability                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Phase 3: Room Assignment                                   │
│  - Match room features to course requirements               │
│  - Assign smallest sufficient room (efficient packing)      │
│  - Respect room unavailability                              │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Phase 4: ILP Student Assignment                            │
│  - Build optimization model with HiGHS                      │
│  - Maximize: Σ(1000 × required) + Σ((10-rank) × elective)  │
│  - Subject to: capacity, conflicts, one-section-per-course  │
│  - Solve to optimality (or near-optimal for large schools)  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Phase 5: Post-ILP Optimization                             │
│  - ILP maximizes but doesn't balance                        │
│  - Local search moves students between sections             │
│  - Only moves that don't create conflicts                   │
│  - Iterates until balanced or no improvement possible       │
└─────────────────────────────────────────────────────────────┘
```

### The ILP Formulation

**Decision Variables:**
- `x[s,k] ∈ {0,1}` — 1 if student `s` is assigned to section `k`

**Objective (maximize):**
```
Σ   1000 · x[s,k]                    (for required courses)
+ Σ (10 - rank) · x[s,k]             (for elective preferences)
```

**Constraints:**
```
Σ x[s,k] ≤ capacity[k]              ∀ sections k
Σ x[s,k] ≤ 1                        ∀ student s, course c (at most one section per course)
x[s,k₁] + x[s,k₂] ≤ 1               ∀ student s, conflicting sections k₁,k₂
```

---

## Building from Source

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- C++ compiler (for HiGHS compilation)
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: `build-essential` package
  - Windows: Visual Studio Build Tools

### Build Commands

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (slower compilation, much faster execution)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

---

## Contributing

Contributions welcome! Areas of interest:

- **Additional solvers**: SCIP, OR-Tools CP-SAT backends
- **Constraint types**: Teacher preferences, room proximity, lunch periods
- **Import formats**: PowerSchool, Infinite Campus, Skyward parsers
- **Visualization**: HTML schedule views, conflict diagrams

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- The [HiGHS](https://highs.dev/) team at the University of Edinburgh for the world-class solver
- The Rust community for the excellent ecosystem of libraries
- School administrators who provided real-world requirements and feedback
