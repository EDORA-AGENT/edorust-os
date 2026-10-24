
import os
import struct

IMAGES = [
    r"build\edora-nova-fixed.img",
    r"build\edora-nova-memory-map.img",
]

PARTITION_LBA = 128
MASK = 0x0FFFFFFF

for path in IMAGES:
    print(f"\n===== {path} =====")

    with open(path, "rb") as f:
        image = f.read()

    part_offset = PARTITION_LBA * 512
    boot = image[part_offset:part_offset + 512]

    bps = struct.unpack_from("<H", boot, 11)[0]
    spc = boot[13]
    reserved = struct.unpack_from("<H", boot, 14)[0]
    fats = boot[16]
    spf = struct.unpack_from("<I", boot, 36)[0]
    root = struct.unpack_from("<I", boot, 44)[0]

    fat_offset = part_offset + reserved * bps
    data_offset = fat_offset + fats * spf * bps
    cluster_size = bps * spc

    def fat_get(c):
        return struct.unpack_from("<I", image, fat_offset + c * 4)[0] & MASK

    def cluster_offset(c):
        return data_offset + (c - 2) * cluster_size

    def get_chain(start):
        result = []
        seen = set()
        c = start
        while c >= 2 and c not in seen:
            seen.add(c)
            result.append(c)
            nxt = fat_get(c)
            if nxt >= 0x0FFFFFF8 or nxt == 0:
                break
            c = nxt
        return result

    def entries(directory_cluster):
        for c in get_chain(directory_cluster):
            off = cluster_offset(c)
            for pos in range(0, cluster_size, 32):
                e = image[off + pos:off + pos + 32]
                if len(e) < 32 or e[0] == 0:
                    return
                if e[0] == 0xE5 or e[11] == 0x0F:
                    continue
                if e[11] & 0x08:
                    continue

                name = e[:8].decode("ascii", "replace").rstrip()
                ext = e[8:11].decode("ascii", "replace").rstrip()
                full_name = name + ("." + ext if ext else "")
                high = struct.unpack_from("<H", e, 20)[0]
                low = struct.unpack_from("<H", e, 26)[0]
                first_cluster = (high << 16) | low
                size = struct.unpack_from("<I", e, 28)[0]
                yield full_name, first_cluster, size

    print("Root directory:")
    boot_cluster = None

    for name, cluster, size in entries(root):
        print(f"  {name:20} cluster={cluster} size={size}")
        if name.upper() == "BOOT":
            boot_cluster = cluster

    if boot_cluster is None:
        print("ERROR: BOOT directory not found")
        continue

    print("BOOT directory:")
    for name, cluster, size in entries(boot_cluster):
        chain = get_chain(cluster) if cluster >= 2 else []
        print(
            f"  {name:20} cluster={cluster} "
            f"size={size} clusters={len(chain)} "
            f"capacity={len(chain) * cluster_size}"
        )
