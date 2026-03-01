# Tessera Work Instruction Entity

This document describes the Work Instruction entity type in Tessera.

## Overview

Work Instructions provide step-by-step procedures for operators. While processes define *what* to do, work instructions define *how* to do it. They capture safety requirements, tools, materials, detailed procedures, and in-process quality checks.

## Entity Type

- **Prefix**: `WORK`
- **File extension**: `.tdt.yaml`
- **Directory**: `manufacturing/work_instructions/`

## Schema

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier (WORK-[26-char ULID]) |
| `title` | string | Short descriptive title (1-200 chars) |
| `status` | enum | `draft`, `review`, `approved`, `released`, `obsolete` |
| `created` | datetime | Creation timestamp (ISO 8601) |
| `author` | string | Author name |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `document_number` | string | Document number (e.g., "WI-MACH-015") |
| `revision` | string | Document revision |
| `description` | string | Purpose/description |
| `safety` | WorkSafety | Safety requirements |
| `tools_required` | array[Tool] | Tools needed |
| `materials_required` | array[Material] | Materials needed |
| `procedure` | array[ProcedureStep] | Step-by-step procedure |
| `quality_checks` | array[QualityCheck] | In-process checks |
| `estimated_duration_minutes` | number | Total estimated time |
| `tags` | array[string] | Tags for filtering |
| `entity_revision` | integer | Entity revision number (default: 1) |

### WorkSafety Object

| Field | Type | Description |
|-------|------|-------------|
| `ppe_required` | array[PpeItem] | Required PPE items |
| `hazards` | array[Hazard] | Hazards and controls |

### PpeItem Object

| Field | Type | Description |
|-------|------|-------------|
| `item` | string | PPE item name |
| `standard` | string | Standard/specification (e.g., "ANSI Z87.1") |

### Hazard Object

| Field | Type | Description |
|-------|------|-------------|
| `hazard` | string | Hazard description |
| `control` | string | Control measure |

### Tool Object

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Tool name |
| `part_number` | string | Part number or specification |

### Material Object

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Material name |
| `specification` | string | Specification or part number |

### ProcedureStep Object

| Field | Type | Description |
|-------|------|-------------|
| `step` | integer | Step number |
| `action` | string | Action to perform |
| `verification` | string | Verification point |
| `caution` | string | Caution/warning |
| `image` | string | Image reference path |
| `estimated_time_minutes` | number | Time for this step |
| `approval` | StepApprovalRequirement | Approval requirements for electronic router (optional) |
| `data_fields` | array[StepDataField] | Data to collect at this step (optional) |
| `equipment` | array[string] | Equipment requiring serial number entry at execution (optional) |

### StepApprovalRequirement Object (Electronic Router)

Defines sign-off requirements for a procedure step. Used for regulated manufacturing (FDA 21 CFR Part 11, ISO 13485) quality hold points.

| Field | Type | Description |
|-------|------|-------------|
| `requires_signoff` | boolean | Whether this step requires sign-off (default: false) |
| `min_approvals` | integer | Minimum number of approvals needed (default: 1) |
| `required_roles` | array[string] | Roles that can approve: "engineering", "quality", "management", "admin" |
| `required_approvers` | array[string] | Specific usernames who must approve |
| `require_signature` | boolean | Require digital signature for 21 CFR Part 11 compliance |
| `quality_hold_point` | boolean | Whether this is a quality hold point (production stops until approved) |

### StepDataField Object (Electronic Router)

Defines data to be collected during step execution (measurements, serial numbers, etc.).

| Field | Type | Description |
|-------|------|-------------|
| `key` | string | Field key for data storage |
| `label` | string | Human-readable label |
| `data_type` | string | Data type: "text", "number", "boolean", "select" (default: "text") |
| `required` | boolean | Whether this field is required (default: false) |
| `units` | string | Units for numeric values (optional) |
| `options` | array[string] | Options for "select" data type |

### QualityCheck Object

| Field | Type | Description |
|-------|------|-------------|
| `at_step` | integer | Step number where check occurs |
| `characteristic` | string | What to check |
| `specification` | string | Specification/tolerance |

### Links

| Field | Type | Description |
|-------|------|-------------|
| `links.process` | EntityId | Parent process |
| `links.controls` | array[EntityId] | Related control plan items |

## Example

```yaml
id: WORK-01KC5B5XKGWKFTTA9YWTGJB9GE
title: "CNC Mill Setup and Operation"
document_number: "WI-MACH-015"
revision: "B"

description: |
  Step-by-step instructions for setting up and operating the
  Haas VF-2 CNC mill for housing machining operation OP-010.

safety:
  ppe_required:
    - item: "Safety Glasses"
      standard: "ANSI Z87.1"
    - item: "Hearing Protection"
      standard: "NRR 25dB minimum"
    - item: "Steel Toe Boots"
      standard: "ASTM F2413"
  hazards:
    - hazard: "Rotating machinery"
      control: "Keep hands clear during operation, use chip brush"
    - hazard: "Sharp edges"
      control: "Wear cut-resistant gloves when handling parts"
    - hazard: "Coolant splash"
      control: "Keep machine doors closed during operation"

tools_required:
  - name: "3/4 inch End Mill"
    part_number: "TL-EM-750"
  - name: "Edge Finder"
    part_number: "TL-EF-001"
  - name: "Torque Wrench"
    part_number: "TL-TW-25"

materials_required:
  - name: "Cutting Coolant"
    specification: "Coolant-500 mixed 8:1"
  - name: "Deburring Tool"
    specification: "Standard"

procedure:
  - step: 1
    action: "Verify correct CNC program loaded: PRG-1234"
    verification: "Program number matches router sheet"
    estimated_time_minutes: 1

  - step: 2
    action: "Load raw material in vise, torque jaw bolts to 25 ft-lbs"
    verification: "Part seated firmly against parallels"
    caution: "Do not over-torque - risk of part distortion"
    image: "images/step2-fixturing.png"
    estimated_time_minutes: 3

  - step: 3
    action: "Touch off work coordinates using edge finder"
    verification: "X0, Y0, Z0 set correctly"
    estimated_time_minutes: 2

  - step: 4
    action: "Verify tool lengths in tool table"
    verification: "All tools measured within 0.001\""
    estimated_time_minutes: 2

  - step: 5
    action: "Run program in single block mode for first part"
    verification: "Observe proper tool paths, no collisions"
    caution: "Keep hand on feed hold button"
    estimated_time_minutes: 20

  - step: 6
    action: "Measure critical dimensions per control plan"
    verification: "All dimensions within specification"
    estimated_time_minutes: 5

  - step: 7
    action: "If acceptable, run production at full speed"
    estimated_time_minutes: 15

  - step: 8
    action: "Deburr all edges"
    verification: "No sharp edges remaining"
    estimated_time_minutes: 2

  # Example step with electronic router features (quality hold point)
  - step: 9
    action: "Final dimensional inspection"
    verification: "All critical dimensions within tolerance"
    estimated_time_minutes: 10
    data_fields:
      - key: bore_diameter
        label: "Bore Diameter (mm)"
        data_type: number
        required: true
        units: mm
      - key: overall_length
        label: "Overall Length (mm)"
        data_type: number
        required: true
        units: mm
      - key: inspector_badge
        label: "Inspector Badge Number"
        data_type: text
        required: true
    equipment:
      - CMM
      - Bore Gauge
    approval:
      requires_signoff: true
      required_roles:
        - quality
      quality_hold_point: true
      require_signature: false

quality_checks:
  - at_step: 6
    characteristic: "Bore Diameter"
    specification: "25.00 +0.025/-0.000 mm"
  - at_step: 6
    characteristic: "Overall Length"
    specification: "100.0 ±0.1 mm"
  - at_step: 8
    characteristic: "Surface Finish"
    specification: "Ra 1.6 μm max"

estimated_duration_minutes: 50

tags: [cnc, milling, housing]
status: released

links:
  process: PROC-01KC5B2GDDQ0JAXFVXYYZ9DWDZ
  controls:
    - CTRL-01KC5B5M87QMYVJT048X27TJ5S

created: 2024-01-15T10:30:00Z
author: John Smith
entity_revision: 2
```

## CLI Commands

### Create a new work instruction

```bash
# Create with default template
tdt work new

# Create with title
tdt work new --title "CNC Mill Setup"

# Create with document number
tdt work new --title "CNC Mill Setup" --doc-number "WI-MACH-015"

# Create linked to a process
tdt work new --title "Mill Setup" --process PROC@1

# Interactive wizard
tdt work new -i

# Create and immediately edit
tdt work new --title "New Work Instruction" --edit
```

### List work instructions

```bash
# List all work instructions
tdt work list

# Filter by process
tdt work list --process PROC@1

# Filter by status
tdt work list --status released

# Search in title/description
tdt work list --search "setup"

# Sort options
tdt work list --sort title
tdt work list --sort doc-number

# Output formats
tdt work list -f json
tdt work list -f csv
tdt work list -f md
```

### Show work instruction details

```bash
# Show by ID
tdt work show WORK-01KC5

# Show using short ID
tdt work show WORK@1

# Output as JSON
tdt work show WORK@1 -f json
```

### Edit a work instruction

```bash
# Open in editor
tdt work edit WORK-01KC5

# Using short ID
tdt work edit WORK@1
```

### Delete or archive a work instruction

```bash
# Permanently delete (checks for incoming links first)
tdt work delete WORK@1

# Force delete even if referenced
tdt work delete WORK@1 --force

# Archive instead of delete (moves to .tdt/archive/)
tdt work archive WORK@1
```

### Manage procedure steps

Add, remove, and list procedure steps from the CLI without editing YAML directly:

```bash
# Add a step (auto-numbered if --step omitted)
tdt work step add WORK@1 --action "Load raw material into fixture"

# Add with verification, time estimate, and caution
tdt work step add WORK@1 --action "Run CNC program" --step 2 \
    --verification "Monitor spindle load, no chatter" \
    --time 45 --caution "Keep hands clear of spindle"

# Add a step requiring quality approval (hold point)
tdt work step add WORK@1 --action "Final dimensional inspection" --step 3 \
    --verification "All dims within tolerance" \
    --require-approval --hold-point

# Add with data collection fields and equipment
tdt work step add WORK@1 --action "Measure bore diameter" --step 4 \
    --verification "Diameter within 25.00 +/-0.025mm" \
    --data-field "bore_diameter:Bore Diameter (mm)" \
    --equipment "Bore Gauge" --require-approval

# List all steps for a work instruction
tdt work step list WORK@1

# Remove a step by number
tdt work step rm WORK@1 --step 3
```

**Step Add Options:**

| Option | Description |
|--------|-------------|
| `--action` | Action description (required) |
| `--step` | Step number (defaults to next available) |
| `--verification` | Verification/check point description |
| `--caution` | Caution/warning note |
| `--time` | Estimated time in minutes |
| `--require-approval` | Mark step as requiring quality approval |
| `--hold-point` | Mark step as a quality hold point |
| `--data-field` | Data field to collect (`key:label` format, repeatable) |
| `--equipment` | Equipment required (repeatable) |

## Best Practices

### Writing Effective Work Instructions

1. **Use active voice** - "Torque the bolt" not "The bolt should be torqued"
2. **One action per step** - Keep steps atomic
3. **Include verification points** - How to know step is complete
4. **Add images** - Reference photos for complex setups
5. **Estimate times** - Help with capacity planning
6. **Safety first** - Document hazards and controls

### Document Numbering

Use a consistent scheme:

```
WI-MACH-001  Machining work instruction #1
WI-ASSY-001  Assembly work instruction #1
WI-INSP-001  Inspection work instruction #1
```

### Cautions and Warnings

Use consistent language:

- **CAUTION**: Risk of equipment damage or minor injury
- **WARNING**: Risk of serious injury
- **DANGER**: Risk of death or severe injury

## Electronic Router / Traveler

Work instructions can define approval requirements and data collection points for use with the LOT entity's electronic router feature. This enables step-level tracking for regulated manufacturing.

### Defining Quality Hold Points

Add approval requirements to steps that need sign-off:

```yaml
procedure:
  - step: 5
    action: "Verify critical dimension"
    verification: "Dimension within tolerance"
    approval:
      requires_signoff: true           # Requires operator sign-off
      required_roles:
        - quality                      # Quality role can approve
      quality_hold_point: true         # Production stops until approved
      require_signature: false         # Digital signature not required
```

### Defining Data Collection Points

Add data fields to steps that need measurements or serial numbers:

```yaml
procedure:
  - step: 6
    action: "Measure bore diameter"
    data_fields:
      - key: bore_diameter
        label: "Bore Diameter (mm)"
        data_type: number
        required: true
        units: mm
      - key: measurement_tool
        label: "Measurement Tool Serial #"
        data_type: text
        required: true
```

### Equipment Traceability

Specify equipment that requires serial number entry at execution:

```yaml
procedure:
  - step: 7
    action: "Torque fasteners to spec"
    equipment:
      - torque_wrench           # Serial number required at execution
      - calibration_adapter
```

### Executing Electronic Router Steps

When a LOT executes steps from this work instruction, operators use:

```bash
# Complete step with required data
tdt lot wi-step LOT@1 --wi WORK@1 --step 5 --complete \
    --data bore_diameter=25.012 \
    --equipment torque_wrench=TW-001

# Approve quality hold point
tdt lot approve LOT@1 --wi WORK@1 --step 5 --role quality \
    --comment "Dimension verified within spec"
```

See the [LOT documentation](lot.md#electronic-router--traveler) for complete electronic router commands.

### Compliance Considerations

For FDA 21 CFR Part 11 compliance:
- Set `require_signature: true` on critical approval steps
- Use `required_approvers` for specific individuals who must approve
- Enable `sign_commits` in manufacturing config for audit trail

For ISO 13485 compliance:
- Define `quality_hold_point: true` for in-process inspection steps
- Use `data_fields` to capture required measurements
- Track equipment serial numbers for traceability

## Validation

```bash
# Validate all project files
tdt validate

# Validate specific file
tdt validate manufacturing/work_instructions/WORK-01KC5B5XKGWKFTTA9YWTGJB9GE.tdt.yaml
```
