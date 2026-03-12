# utils.py
#
# Small helper functions used across the project.
# Keeping these here avoids repeating code in other files.


def print_divider(label=None, width=60):
    """Print a horizontal divider, optionally with a centered label."""
    if label:
        line = f"  {label}  "
        print(line.center(width, "="))
    else:
        print("=" * width)


def print_header(title):
    """Print a section header with dividers above and below."""
    print_divider()
    print(f"  {title}")
    print_divider()


def numbered_menu(options):
    """
    Print a numbered list of options and return the user's choice (0-indexed).

    Parameters
    ----------
    options : list of str

    Returns
    -------
    int  — index of chosen item, or -1 if input was invalid
    """
    for i, option in enumerate(options):
        print(f"  [{i + 1}] {option}")

    raw = input("\nEnter number: ").strip()

    if not raw.isdigit():
        return -1

    choice = int(raw) - 1  # Convert to 0-based index

    if 0 <= choice < len(options):
        return choice

    return -1


def confirm(prompt="Are you sure? (y/n): "):
    """
    Ask the user a yes/no question.
    Returns True for 'y', False for anything else.
    """
    answer = input(prompt).strip().lower()
    return answer == "y"


def format_unit_line(unit, mct_mp=None, sp_override=None):
    mp_display  = mct_mp if mct_mp is not None else unit.base_mp
    sp_display  = sp_override if sp_override is not None else unit.current_sp
    reduced_tag = " [REDUCED]" if unit.is_reduced else ""
    police_tag  = " [POLICE]"  if unit.police     else ""
    loc         = unit.location if unit.location   else "Off Map"

    return (
        f"  {unit.name:<26}"
        f" SP:{sp_display}"
        f" MP:{mp_display}"
        f"  @ {loc}"
        f"{reduced_tag}{police_tag}"
    )