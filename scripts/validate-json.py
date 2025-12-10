#!/usr/bin/env python3
"""
Validate FEA JSON data files for schema compliance and referential integrity.
LLMs are fast but not great at structured validation - this tool does it right.

Usage: python scripts/validate-json.py local-data/fea/
"""

import json
import sys
from pathlib import Path
from typing import Any

def load_json(path: Path) -> Any:
    with open(path) as f:
        return json.load(f)

def validate_students(students: list, course_ids: set) -> list[str]:
    errors = []
    seen_ids = set()
    for s in students:
        sid = s.get("id", "MISSING")
        if sid in seen_ids:
            errors.append(f"Duplicate student ID: {sid}")
        seen_ids.add(sid)

        if not s.get("name"):
            errors.append(f"Student {sid}: missing name")
        if not isinstance(s.get("grade"), int) or not (9 <= s.get("grade", 0) <= 12):
            errors.append(f"Student {sid}: invalid grade {s.get('grade')}")

        for cid in s.get("required_courses", []):
            if cid not in course_ids:
                errors.append(f"Student {sid}: unknown required course '{cid}'")
        for cid in s.get("elective_preferences", []):
            if cid not in course_ids:
                errors.append(f"Student {sid}: unknown elective '{cid}'")
    return errors

def validate_teachers(teachers: list, course_ids: set) -> list[str]:
    errors = []
    seen_ids = set()
    for t in teachers:
        tid = t.get("id", "MISSING")
        if tid in seen_ids:
            errors.append(f"Duplicate teacher ID: {tid}")
        seen_ids.add(tid)

        if not t.get("name"):
            errors.append(f"Teacher {tid}: missing name")
        if "lovelace" in t.get("name", "").lower():
            errors.append(f"Teacher {tid}: contains 'Lovelace' - should be Edwards")

        for cid in t.get("subjects", []):
            if cid not in course_ids:
                errors.append(f"Teacher {tid}: unknown subject '{cid}'")
    return errors

def validate_courses(courses: list) -> tuple[set, list[str]]:
    errors = []
    course_ids = set()
    for c in courses:
        cid = c.get("id", "MISSING")
        if cid in course_ids:
            errors.append(f"Duplicate course ID: {cid}")
        course_ids.add(cid)

        if not c.get("name"):
            errors.append(f"Course {cid}: missing name")
        if not isinstance(c.get("max_students"), int) or c.get("max_students", 0) <= 0:
            errors.append(f"Course {cid}: invalid max_students")
        if not isinstance(c.get("sections"), int) or c.get("sections", 0) <= 0:
            errors.append(f"Course {cid}: invalid sections count")
    return course_ids, errors

def validate_rooms(rooms: list) -> list[str]:
    errors = []
    seen_ids = set()
    for r in rooms:
        rid = r.get("id", "MISSING")
        if rid in seen_ids:
            errors.append(f"Duplicate room ID: {rid}")
        seen_ids.add(rid)

        if not isinstance(r.get("capacity"), int) or r.get("capacity", 0) <= 0:
            errors.append(f"Room {rid}: invalid capacity")
    return errors

def main(data_dir: str):
    path = Path(data_dir)
    all_errors = []

    # Load and validate courses first (needed for references)
    courses_file = path / "courses.json"
    if not courses_file.exists():
        print(f"ERROR: {courses_file} not found")
        sys.exit(1)

    courses = load_json(courses_file)
    course_ids, course_errors = validate_courses(courses)
    all_errors.extend(course_errors)

    # Validate other files
    for filename, validator in [
        ("students.json", lambda d: validate_students(d, course_ids)),
        ("teachers.json", lambda d: validate_teachers(d, course_ids)),
        ("rooms.json", validate_rooms),
    ]:
        filepath = path / filename
        if filepath.exists():
            data = load_json(filepath)
            all_errors.extend(validator(data))
        else:
            all_errors.append(f"Missing file: {filename}")

    # Report results
    if all_errors:
        print(f"VALIDATION FAILED - {len(all_errors)} errors:\n")
        for err in all_errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        print(f"VALIDATION PASSED")
        print(f"  Courses: {len(courses)}")
        print(f"  Course IDs: {sorted(course_ids)}")
        sys.exit(0)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <data-directory>")
        sys.exit(1)
    main(sys.argv[1])
