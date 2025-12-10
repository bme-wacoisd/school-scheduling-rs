#!/usr/bin/env python3
"""
Build scheduler JSON files from simple text/CSV formats.
LLMs extract data poorly into complex JSON - this tool enforces structure.

Usage:
  python scripts/build-json.py courses local-data/fea/courses.txt -o local-data/fea/courses.json
  python scripts/build-json.py students local-data/fea/students.txt -o local-data/fea/students.json
  python scripts/build-json.py teachers local-data/fea/teachers.txt -o local-data/fea/teachers.json
  python scripts/build-json.py rooms local-data/fea/rooms.txt -o local-data/fea/rooms.json
"""

import argparse
import json
import re
import sys
from pathlib import Path


def parse_courses(lines: list[str]) -> list[dict]:
    """
    Input format (one course per line):
      id | name | max_students | grades | features | sections

    Examples:
      ip | Instructional Practices | 25 | 9,10,11,12 | | 2
      practicum | Practicum | 20 | 11,12 | | 1
      math-alg1 | Algebra 1 | 25 | 9 | | 1
      sci-bio | Biology | 24 | 10 | lab | 1

    grades: comma-separated list, or "all" for no restriction
    features: comma-separated list, or empty
    """
    courses = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 6:
            print(f"WARN: Skipping malformed line: {line}", file=sys.stderr)
            continue

        cid, name, max_str, grades_str, features_str, sections_str = parts[:6]

        # Parse grades
        grades = None
        if grades_str and grades_str.lower() != 'all':
            grades = [int(g.strip()) for g in grades_str.split(',') if g.strip()]

        # Parse features
        features = [f.strip() for f in features_str.split(',') if f.strip()]

        courses.append({
            "id": cid,
            "name": name,
            "max_students": int(max_str),
            "grade_restrictions": grades,
            "required_features": features,
            "sections": int(sections_str)
        })
    return courses


def parse_students(lines: list[str]) -> list[dict]:
    """
    Input format (one student per line):
      id | name | grade | required_courses | elective_preferences

    Examples:
      s001 | Alice Johnson | 10 | ip,practicum,math-alg1 | art,music
      s002 | Bob Smith | 11 | ip,practicum,eng11 | pe

    courses: comma-separated course IDs
    """
    students = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 5:
            print(f"WARN: Skipping malformed line: {line}", file=sys.stderr)
            continue

        sid, name, grade_str, required_str, electives_str = parts[:5]

        required = [c.strip() for c in required_str.split(',') if c.strip()]
        electives = [c.strip() for c in electives_str.split(',') if c.strip()]

        students.append({
            "id": sid,
            "name": name,
            "grade": int(grade_str),
            "required_courses": required,
            "elective_preferences": electives
        })
    return students


def parse_teachers(lines: list[str]) -> list[dict]:
    """
    Input format (one teacher per line):
      id | name | subjects | max_sections | unavailable

    Examples:
      edwards | Brian Edwards | ip,practicum,comm-tech | 4 |
      stone | Kelley Stone | eng10,eng11,pedagogy | 5 |
      pasisis | Gina Pasisis | eng9,pedagogy | 2 | Mon:1,Mon:2

    subjects: comma-separated course IDs
    unavailable: comma-separated Day:Period pairs, or empty
    """
    teachers = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 5:
            print(f"WARN: Skipping malformed line: {line}", file=sys.stderr)
            continue

        tid, name, subjects_str, max_str, unavail_str = parts[:5]

        # Check for Lovelace
        if 'lovelace' in name.lower():
            print(f"ERROR: Found 'Lovelace' in teacher name - should be Edwards", file=sys.stderr)
            sys.exit(1)

        subjects = [s.strip() for s in subjects_str.split(',') if s.strip()]

        unavailable = []
        if unavail_str:
            for slot in unavail_str.split(','):
                slot = slot.strip()
                if ':' in slot:
                    day, period = slot.split(':')
                    unavailable.append({"day": day.strip(), "slot": int(period.strip())})

        teachers.append({
            "id": tid,
            "name": name,
            "subjects": subjects,
            "max_sections": int(max_str),
            "unavailable": unavailable
        })
    return teachers


def parse_rooms(lines: list[str]) -> list[dict]:
    """
    Input format (one room per line):
      id | name | capacity | features | unavailable

    Examples:
      room1 | Room 1 (Stone) | 25 | |
      room2 | Room 2 (Futral) | 25 | |
      room3 | Room 3 (Edwards) | 25 | |
      room4 | Room 4 (Shared) | 20 | |

    features: comma-separated list (lab, gym, etc), or empty
    unavailable: comma-separated Day:Period pairs, or empty
    """
    rooms = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 5:
            print(f"WARN: Skipping malformed line: {line}", file=sys.stderr)
            continue

        rid, name, cap_str, features_str, unavail_str = parts[:5]

        features = [f.strip() for f in features_str.split(',') if f.strip()]

        unavailable = []
        if unavail_str:
            for slot in unavail_str.split(','):
                slot = slot.strip()
                if ':' in slot:
                    day, period = slot.split(':')
                    unavailable.append({"day": day.strip(), "slot": int(period.strip())})

        rooms.append({
            "id": rid,
            "name": name,
            "capacity": int(cap_str),
            "features": features,
            "unavailable": unavailable
        })
    return rooms


def main():
    parser = argparse.ArgumentParser(description='Build scheduler JSON from simple text formats')
    parser.add_argument('type', choices=['courses', 'students', 'teachers', 'rooms'])
    parser.add_argument('input', help='Input text file')
    parser.add_argument('-o', '--output', help='Output JSON file (default: stdout)')
    args = parser.parse_args()

    with open(args.input) as f:
        lines = f.readlines()

    parsers = {
        'courses': parse_courses,
        'students': parse_students,
        'teachers': parse_teachers,
        'rooms': parse_rooms,
    }

    data = parsers[args.type](lines)
    json_str = json.dumps(data, indent=2)

    if args.output:
        Path(args.output).write_text(json_str)
        print(f"Wrote {len(data)} {args.type} to {args.output}")
    else:
        print(json_str)


if __name__ == '__main__':
    main()
