# FEA Scheduler Initialization

You are initializing a scheduling session for Waco ISD Future Educators Academy.

## Session Setup Protocol

1. **Verify environment**
   ```bash
   pwd && cargo build --release 2>&1 | tail -5
   ```

2. **Check existing progress**
   - Read `local-data/fea/progress.json` if it exists
   - Read recent git log: `git log --oneline -5`

3. **Assess data state**
   - Check `local-data/fea/` for existing JSON files
   - If missing, extract from `input/fea/*.pdf` using structured parsing

4. **Create progress tracker** (if new session)
   Write to `local-data/fea/progress.json`:
   ```json
   {
     "session_start": "<timestamp>",
     "data_extracted": false,
     "schedule_generated": false,
     "validation_passed": false,
     "features": []
   }
   ```

## Data Extraction (if needed)

Extract from PDFs in `input/fea/`:
- `master.pdf` - Master schedule reference
- `cohorts.pdf` - Student cohort overview
- `cohort-1.pdf` through `cohort-4.pdf` - Individual student lists
- Reference `input/fea/teachers.md` for teacher data

### FEA Constraints (from teachers.md)
- 4 classrooms total (Pasisis & Giddings share one)
- Multi-grade classes are normal
- Mixed-subject sections allowed (e.g., Instructional Practices + Practicum together)
- Part-time teachers: Pasisis, Giddings (retired, limited hours)

### Teacher ID Mapping
| Name | ID | Subjects |
|------|-----|----------|
| Kelley Stone | stone | ELA, pedagogy, time management |
| Trent Futral | futral | Math |
| Brian Edwards | edwards | Instructional Practices, Practicum, Communications & Technology |
| Gina Pasisis | pasisis | ELA, pedagogy (part-time) |
| Susan Giddings | giddings | ELA, pedagogy (part-time) |

**CRITICAL**: No references to "Lovelace" in output - Edwards replaced her.

## Handoff

After initialization, write a summary to `local-data/fea/progress.json` and commit:
```bash
git add local-data/fea/ && git commit -m "FEA: Initialize session data"
```

Then report what was accomplished and what the next worker session should tackle.
