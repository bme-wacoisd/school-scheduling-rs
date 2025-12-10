# FEA Worker Session

You are continuing work on Waco ISD Future Educators Academy scheduling.

## Session Start Protocol

1. **Orient yourself**
   ```bash
   pwd
   git log --oneline -3
   ```

2. **Load progress state**
   - Read `local-data/fea/progress.json`
   - Identify the highest-priority incomplete feature

3. **Verify baseline** (if schedule exists)
   ```bash
   cargo run --release -- validate --schedule ./output/fea/schedule.json --data ./local-data/fea -v
   ```

## Work Rules

- **One feature per session** - Complete fully before moving on
- **Commit after each feature** - Leave clear trail for next session
- **Update progress.json** - Mark completed features, add discovered issues
- **Never remove tests** - Only add or fix, never delete validation checks

## Feature Priority Order

1. Data completeness (all students, teachers, courses, rooms in JSON)
2. Schedule generation without hard constraint violations
3. Teacher schedule balance (no overloads)
4. Student required course fulfillment (100% target)
5. Elective preference satisfaction
6. Room utilization optimization

## Validation Checkpoints

After any schedule change, run:
```bash
cargo run --release -- validate --schedule ./output/fea/schedule.json --data ./local-data/fea -v
```

**Hard constraints must pass.** Soft scores are optimization targets.

## Session End Protocol

1. Update `local-data/fea/progress.json` with:
   - What was completed
   - What issues were discovered
   - Recommended next task

2. Commit with descriptive message:
   ```bash
   git add -A && git commit -m "FEA: <what was accomplished>"
   ```

3. Report to user what was done and what remains
