#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
root="$(cd "${root}" && pwd)"
errors=0
warnings=0

error() {
  printf 'ERROR: %s\n' "$1" >&2
  errors=$((errors + 1))
}

warn() {
  printf 'WARNING: %s\n' "$1" >&2
  warnings=$((warnings + 1))
}

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

field_value() {
  local file="$1"
  local wanted="$2"
  awk -F'|' -v wanted="$wanted" '
    function trim(value) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      return value
    }
    /^\|/ {
      key = trim($2)
      if (key == wanted) {
        print trim($3)
        exit
      }
    }
  ' "$file"
}

has_placeholder() {
  [[ "$1" == *'<'*'>'* ]] || [[ "$1" == *'{"'* ]] || [[ "$1" == *'{ '* ]]
}

is_date() {
  [[ "$1" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]
}

owner_status_is_active() {
  local file="$1"
  grep -Eiq \
    '^[[:space:]]*>[[:space:]]*Document status:[[:space:]]*(Active|Review)|^[[:space:]]*\*\*Status\*\*:[[:space:]]*(In Progress|Review)|^[[:space:]]*Status:[[:space:]]*(In Progress|Review)' \
    "$file"
}

owner_status_is_closed() {
  local file="$1"
  grep -Eiq \
    '^[[:space:]]*>[[:space:]]*Document status:[[:space:]]*(Complete|Completed|Cancelled)|^[[:space:]]*\*\*Status\*\*:[[:space:]]*(Complete|Completed|Cancelled)|^[[:space:]]*Status:[[:space:]]*(Complete|Completed|Done|Cancelled)' \
    "$file"
}

owner_status_is_complete() {
  local file="$1"
  grep -Eiq \
    '^[[:space:]]*>[[:space:]]*Document status:[[:space:]]*(Complete|Completed)|^[[:space:]]*\*\*Status\*\*:[[:space:]]*(Complete|Completed)|^[[:space:]]*Status:[[:space:]]*(Complete|Completed|Done)' \
    "$file"
}

validate_required_file_text() {
  local relative="$1"
  shift
  local file="$root/$relative"
  if [[ ! -f "$file" ]]; then
    error "missing collaboration governance file: $relative"
    return
  fi
  local required
  for required in "$@"; do
    if ! grep -Fq "$required" "$file"; then
      error "$relative is missing required collaboration text: $required"
    fi
  done
}

validate_claim_record() {
  local file="$1"
  local relative="${file#"$root/"}"
  local count
  count="$(grep -Ec '^## Collaboration Claim[[:space:]]*$' "$file" || true)"

  if [[ "$count" -eq 0 ]]; then
    return
  fi
  if [[ "$count" -ne 1 ]]; then
    error "$relative must contain exactly one effective Collaboration Claim section"
    return
  fi

  local fields=(
    "Claim State"
    "Responsible Actor"
    "Executing Agent"
    "Work Slice"
    "Claimed At"
    "Source Issue"
    "Governance Claim PR"
    "Authorization Mode"
    "Authorization Evidence"
    "Implementation PR"
    "Last Updated"
    "Handoff / Release Condition"
  )
  local field value
  for field in "${fields[@]}"; do
    value="$(field_value "$file" "$field")"
    if [[ -z "$value" ]]; then
      error "$relative Collaboration Claim is missing field: $field"
    fi
  done

  local state actor agent slice claimed_at source_issue claim_ref auth_mode auth_evidence implementation_pr last_updated
  state="$(field_value "$file" "Claim State")"
  actor="$(field_value "$file" "Responsible Actor")"
  agent="$(field_value "$file" "Executing Agent")"
  slice="$(field_value "$file" "Work Slice")"
  claimed_at="$(field_value "$file" "Claimed At")"
  source_issue="$(field_value "$file" "Source Issue")"
  claim_ref="$(field_value "$file" "Governance Claim PR")"
  auth_mode="$(field_value "$file" "Authorization Mode")"
  auth_evidence="$(field_value "$file" "Authorization Evidence")"
  implementation_pr="$(field_value "$file" "Implementation PR")"
  last_updated="$(field_value "$file" "Last Updated")"

  case "$state" in
    Unclaimed|Claimed|"Handoff Pending"|Released|Closed) ;;
    "Claim Pending")
      error "$relative persists Claim Pending; pending is an open-PR derived state only"
      ;;
    *)
      error "$relative has unsupported Claim State: ${state:-<empty>}"
      ;;
  esac

  if [[ -n "$claimed_at" && "$claimed_at" != "Not applicable" ]] && ! is_date "$claimed_at"; then
    error "$relative Claimed At must use YYYY-MM-DD or Not applicable"
  fi
  if [[ -n "$last_updated" ]] && ! is_date "$last_updated"; then
    error "$relative Last Updated must use YYYY-MM-DD"
  fi

  case "$state" in
    Claimed|"Handoff Pending"|Closed)
      for value in "$actor" "$agent" "$slice" "$claimed_at" "$claim_ref" "$auth_mode" "$auth_evidence" "$last_updated"; do
        if [[ -z "$value" ]] || has_placeholder "$value"; then
          error "$relative has an incomplete persistent $state claim value: ${value:-<empty>}"
        fi
      done

      if [[ ! "$actor" =~ ^@[A-Za-z0-9_-]+$ ]]; then
        error "$relative Responsible Actor must be a GitHub-style @login"
      fi
      if ! is_date "$claimed_at"; then
        error "$relative persistent $state claim requires a concrete Claimed At date"
      fi
      if [[ ! "$claim_ref" =~ ^#[0-9]+$ && ! "$claim_ref" =~ ^Direct[[:space:]]commit[[:space:]][0-9a-fA-F]{7,40}$ ]]; then
        error "$relative Governance Claim PR must be #NN or Direct commit <SHA>"
      fi
      case "$auth_mode" in
        "Independent review"|"Single-maintainer merge"|"Direct commit"|"Emergency override") ;;
        *) error "$relative has unsupported Authorization Mode: ${auth_mode:-<empty>}" ;;
      esac
      if [[ "$claim_ref" =~ ^Direct[[:space:]]commit ]] && [[ "$auth_mode" != "Direct commit" && "$auth_mode" != "Emergency override" ]]; then
        error "$relative direct-commit claim reference requires Direct commit or Emergency override authorization"
      fi
      if [[ "$claim_ref" =~ ^#[0-9]+$ ]] && [[ "$auth_mode" == "Direct commit" ]]; then
        error "$relative PR-backed claim cannot use Direct commit authorization"
      fi
      ;;
  esac

  if [[ "$state" == "Closed" ]]; then
    if ! owner_status_is_closed "$file"; then
      error "$relative Claim State Closed does not match a Complete or Cancelled delivery state"
    elif owner_status_is_complete "$file" && ! grep -Eiq 'Completion Commit(s)?:[[:space:]]*`?[0-9a-fA-F]{7,40}' "$file"; then
      error "$relative closed Complete owner lacks Completion Commit evidence"
    fi
  fi

  if [[ "$state" == "Unclaimed" ]]; then
    if [[ "$claim_ref" != "Not applicable" && "$claim_ref" != "Pending" ]]; then
      warn "$relative is Unclaimed but has Governance Claim PR '$claim_ref'"
    fi
  fi

  if [[ -z "$source_issue" || -z "$implementation_pr" ]]; then
    error "$relative must explicitly record Source Issue and Implementation PR, using None/Not started when applicable"
  fi
}

resolve_diff_base() {
  if [[ -n "${COLLABORATION_VALIDATION_BASE:-}" ]] && git -C "$root" rev-parse --verify -q "${COLLABORATION_VALIDATION_BASE}^{commit}" >/dev/null; then
    printf '%s' "$COLLABORATION_VALIDATION_BASE"
    return
  fi

  if [[ -n "${GITHUB_BASE_REF:-}" ]] && git -C "$root" rev-parse --verify -q "origin/${GITHUB_BASE_REF}^{commit}" >/dev/null; then
    git -C "$root" merge-base HEAD "origin/${GITHUB_BASE_REF}"
    return
  fi

  if git -C "$root" rev-parse --verify -q 'origin/main^{commit}' >/dev/null && [[ "$(git -C "$root" rev-parse HEAD)" != "$(git -C "$root" rev-parse origin/main)" ]]; then
    git -C "$root" merge-base HEAD origin/main
    return
  fi

  git -C "$root" rev-parse HEAD^ 2>/dev/null || git -C "$root" rev-parse HEAD
}

validate_changed_active_owners() {
  local base="$1"
  local changed
  changed="$(git -C "$root" diff --name-only "$base" HEAD -- docs/iterations docs/tasks docs/backlog/active 2>/dev/null || true)"
  [[ -n "$changed" ]] || return 0

  local relative file
  while IFS= read -r relative; do
    [[ -n "$relative" ]] || continue
    case "$relative" in
      docs/iterations/README.md|docs/iterations/TEMPLATE.md) continue ;;
      *.md) ;;
      *) continue ;;
    esac
    file="$root/$relative"
    [[ -f "$file" ]] || continue

    if owner_status_is_active "$file" && ! grep -Eq '^## Collaboration Claim[[:space:]]*$' "$file"; then
      error "$relative is an active/review owner changed after adoption but lacks a Collaboration Claim"
    fi
  done <<< "$changed"
}

validate_required_file_text \
  docs/sop/AGENT-COLLABORATION.md \
  "Adoption And Migration" \
  "Persistent Claim Model" \
  "Single-maintainer merge" \
  "Emergency Override" \
  "Mandatory Merge-Time CAS Preflight"
validate_required_file_text \
  docs/iterations/TEMPLATE.md \
  "## Collaboration Claim" \
  "Work Slice" \
  "Authorization Mode"
validate_required_file_text \
  docs/sop/REQUIREMENT-INTAKE.md \
  "Collaboration Claim" \
  "Work Slice"
validate_required_file_text \
  docs/sop/START-ITERATION.md \
  "effective Collaboration Claim" \
  "merge-time CAS"
validate_required_file_text \
  docs/sop/LONG-RUNNING-TASK.md \
  "Collaboration Claim" \
  "Authorization Mode"
validate_required_file_text \
  docs/sop/DOC-CHECK.md \
  "validate_collaboration_claims.sh"
validate_required_file_text \
  scripts/release_preflight.sh \
  "validate_collaboration_claims.sh"

owner_files=""
for folder in "$root/docs/iterations" "$root/docs/tasks" "$root/docs/backlog/active"; do
  if [[ -d "$folder" ]]; then
    owner_files+="$(find "$folder" -type f -name '*.md' ! -path '*/iterations/README.md' ! -path '*/iterations/TEMPLATE.md' -print)"$'\n'
  fi
done

while IFS= read -r file; do
  [[ -n "$file" ]] || continue
  validate_claim_record "$file"
done <<< "$owner_files"

base="$(resolve_diff_base)"
validate_changed_active_owners "$base"

if [[ "$errors" -gt 0 ]]; then
  printf 'Collaboration claim validation failed: %d error(s), %d warning(s).\n' "$errors" "$warnings" >&2
  exit 1
fi

printf 'Collaboration claim validation passed: %d warning(s).\n' "$warnings"
