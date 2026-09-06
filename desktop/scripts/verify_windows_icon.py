#!/usr/bin/env python3
"""Check the built PE icon resource against the source ICO without running it."""
import argparse
from pathlib import Path
import struct

import pefile

ASSETS = Path(__file__).resolve().parents[1] / "assets"


def verify_windows_icon(executable, icon=ASSETS / "icon.ico"):
    source = Path(icon).read_bytes()
    reserved, kind, count = struct.unpack_from("<HHH", source)
    if (reserved, kind) != (0, 1) or count == 0:
        raise ValueError(f"Invalid source icon: {icon}")
    expected = {}
    for i in range(count):
        width, height, _, _, planes, bits, length, offset = struct.unpack_from(
            "<BBBBHHII", source, 6 + i * 16)
        # ICO permits zero planes; Windows resource compilers normalize it to 1.
        expected[(width or 256, height or 256, planes or 1, bits)] = source[offset:offset + length]

    with pefile.PE(str(executable), fast_load=True) as pe:
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_RESOURCE"]])
        resources = {}
        root = getattr(pe, "DIRECTORY_ENTRY_RESOURCE", None)
        for resource_type in root.entries if root else []:
            for name in resource_type.directory.entries:
                for language in name.directory.entries:
                    entry = language.data.struct
                    resources[(resource_type.id, name.id, language.id)] = pe.get_data(
                        entry.OffsetToData, entry.Size)
        groups = [(language, data) for (kind, name, language), data in resources.items()
                  if kind == 14 and name == 1]  # RT_GROUP_ICON, GPUI's icon ID
        if not groups:
            raise ValueError(f"{executable}: missing Windows icon resource 1")
        for language, group in groups:
            reserved, kind, count = struct.unpack_from("<HHH", group)
            if (reserved, kind, count) != (0, 1, len(expected)):
                raise ValueError("Embedded icon group does not match the source ICO")
            found = set()
            for i in range(count):
                width, height, _, _, planes, bits, length, resource_id = struct.unpack_from(
                    "<BBBBHHIH", group, 6 + i * 14)
                key = (width or 256, height or 256, planes or 1, bits)
                actual = resources.get((3, resource_id, language))  # RT_ICON
                if key not in expected or actual != expected[key] or len(actual) != length:
                    raise ValueError(f"Missing or stale embedded icon image: {key[:2]}")
                found.add(key)
            if found != expected.keys():
                raise ValueError("Embedded icon is missing source sizes")
    sizes = ", ".join(str(width) for width in sorted({key[0] for key in expected}))
    print(f"Verified {executable}: icon resource 1 matches source ({sizes} px)")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    args = parser.parse_args()
    try:
        verify_windows_icon(args.executable)
    except (ValueError, pefile.PEFormatError) as error:
        raise SystemExit(str(error)) from error
