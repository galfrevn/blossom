#!/bin/sh
set -eu

report_against_budget() {
  name=$1
  measured_bytes=$2
  limit_bytes=$3
  unit=$4
  LC_ALL=C awk -v name="$name" -v measured="$measured_bytes" -v limit="$limit_bytes" -v unit="$unit" \
    'BEGIN { bytes_per_unit = unit == "MB" ? 1000000 : 1000; printf "%s: %.2f %s of %.2f %s\n", name, measured / bytes_per_unit, unit, limit / bytes_per_unit, unit }'
  if [ "$measured_bytes" -gt "$limit_bytes" ]; then
    echo "$name is over its budget in documentation/adr/architecture/budgets.md" >&2
    exit 1
  fi
}

case "${1:-}" in
  javascript)
    if [ ! -d ui/dist ]; then
      echo "there is no ui/dist: run bun run --cwd ui build first" >&2
      exit 2
    fi
    measured_bytes=$(find ui/dist -name '*.js' -exec gzip -9 -c {} \; | wc -c | tr -d ' ')
    report_against_budget "the ui's javascript, gzipped" "$measured_bytes" 250000 KB
    ;;
  installer)
    disk_image_folder=target/release/bundle/dmg
    disk_image_count=$(find "$disk_image_folder" -name '*.dmg' 2>/dev/null | wc -l | tr -d ' ')
    if [ "$disk_image_count" -ne 1 ]; then
      echo "expected one disk image in $disk_image_folder, found $disk_image_count: run bun run build first" >&2
      exit 2
    fi
    disk_image=$(find "$disk_image_folder" -name '*.dmg')
    measured_bytes=$(wc -c < "$disk_image" | tr -d ' ')
    report_against_budget "the disk image" "$measured_bytes" 15000000 MB
    ;;
  time)
    job_name=${2:-}
    budget_seconds=${3:-}
    if [ -z "$job_name" ] || [ -z "$budget_seconds" ]; then
      echo "usage: sh budgets.sh time <job> <budget in seconds>" >&2
      exit 2
    fi
    if [ -z "${JOB_STARTED_AT:-}" ]; then
      echo "JOB_STARTED_AT is not set: the job's first step has to record it" >&2
      exit 2
    fi
    elapsed_seconds=$(($(date +%s) - JOB_STARTED_AT))
    cache_state="cold, the rust cache did not match in full"
    if [ "${RUST_CACHE_FULL_MATCH:-}" = "true" ]; then
      cache_state="warm"
    fi
    result="$job_name: $elapsed_seconds s of $budget_seconds s, $cache_state"
    echo "$result"
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
      echo "$result" >> "$GITHUB_STEP_SUMMARY"
    fi
    if [ "$cache_state" = "warm" ] && [ "$elapsed_seconds" -gt "$budget_seconds" ]; then
      echo "$job_name is over its time budget in documentation/adr/architecture/pipeline.md" >&2
      exit 1
    fi
    ;;
  *)
    echo "usage: sh budgets.sh javascript|installer|time" >&2
    exit 2
    ;;
esac
