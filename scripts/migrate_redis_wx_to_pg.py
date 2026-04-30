#!/usr/bin/env python3
"""
One-time migration: copy WeChat follower data from Redis → PostgreSQL.

Usage:
  # moicen
  python3 migrate_redis_wx_to_pg.py postgres://postgres:postgres@localhost:5432/htyuc_moicen

  # alchemy (huiwings)
  python3 migrate_redis_wx_to_pg.py postgres://postgres:postgres@localhost:5432/htyuc_huiwings

Reads HW_ALL_USER_OPENIDS_{app_id} and HW_OPENID_INFO_{openid}_{app_id} from Redis (localhost:6379)
and upserts into wx_followers / wx_follower_infos tables.
"""

import json
import subprocess
import sys
from datetime import datetime, timezone

REDIS_HOST = "127.0.0.1"
REDIS_PORT = "6379"

RUN_ALL_OPENIDS = True
RUN_OPENID_INFO = True


def redis_cli(*args):
    result = subprocess.run(
        ["redis-cli", "-h", REDIS_HOST, "-p", REDIS_PORT, *args],
        capture_output=True,
        text=True,
        timeout=300,
    )
    if result.returncode != 0:
        print(f"redis-cli error: {result.stderr}", file=sys.stderr)
        return None
    return result.stdout.rstrip("\n")


def psql_exec(db_url, sql, params=None):
    """Execute SQL via psql.  Returns stdout."""
    cmd = ["psql", db_url, "--tuples-only", "--no-align", "-c", sql]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    if result.returncode != 0:
        print(f"psql error: {result.stderr}", file=sys.stderr)
        return None
    return result.stdout


def get_all_openids_keys():
    output = redis_cli("KEYS", "HW_ALL_USER_OPENIDS_*")
    if not output:
        return []
    return output.splitlines()


def get_openid_info_keys():
    output = redis_cli("KEYS", "HW_OPENID_INFO_*")
    if not output:
        return []
    return output.splitlines()


def parse_openids_key(key):
    """Extract app_id from 'HW_ALL_USER_OPENIDS_{app_id}'."""
    prefix = "HW_ALL_USER_OPENIDS_"
    return key[len(prefix):]


def parse_openid_info_key(key):
    """Extract (openid, app_id) from 'HW_OPENID_INFO_{openid}_{app_id}'."""
    prefix = "HW_OPENID_INFO_"
    suffix = key[len(prefix):]
    # openid can contain underscores, app_id also can.
    # Strategy: the app_id is always a UUID (hex with dashes), so find the last UUID.
    parts = suffix.rsplit("_", 1)
    if len(parts) == 2:
        return parts[0], parts[1]
    return suffix, ""


def migrate_openids(db_url):
    """Migrate HW_ALL_USER_OPENIDS_* → wx_followers."""
    keys = get_all_openids_keys()
    print(f"Found {len(keys)} ALL_USER_OPENIDS keys")

    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")
    total_followers = 0

    for key in keys:
        app_id = parse_openids_key(key)
        raw = redis_cli("GET", key)
        if not raw:
            print(f"  WARN: empty value for {key}, skipping")
            continue

        try:
            openids = json.loads(raw)
        except json.JSONDecodeError as e:
            print(f"  WARN: invalid JSON for {key}: {e}, skipping")
            continue

        if not isinstance(openids, list):
            print(f"  WARN: value is not a list for {key} (type={type(openids).__name__}), skipping")
            continue

        for openid in openids:
            if not openid or not isinstance(openid, str):
                continue
            escaped_openid = openid.replace("'", "''")
            sql = (
                f"INSERT INTO wx_followers (app_id, openid, refreshed_at) "
                f"VALUES ('{app_id}', '{escaped_openid}', '{now}') "
                f"ON CONFLICT (app_id, openid) DO UPDATE SET refreshed_at = EXCLUDED.refreshed_at;"
            )
            psql_exec(db_url, sql)
            total_followers += 1

        print(f"  {key}: {len(openids)} followers")

    print(f"\nTotal wx_followers migrated: {total_followers}")


def migrate_openid_infos(db_url):
    """Migrate HW_OPENID_INFO_* → wx_follower_infos."""
    keys = get_openid_info_keys()
    print(f"Found {len(keys)} OPENID_INFO keys")

    total_infos = 0
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")

    for key in keys:
        openid, app_id = parse_openid_info_key(key)
        if not app_id:
            print(f"  WARN: could not parse app_id from {key}, skipping")
            continue

        raw = redis_cli("GET", key)
        if not raw:
            print(f"  WARN: empty value for {key}, skipping")
            continue

        try:
            info = json.loads(raw)
        except json.JSONDecodeError as e:
            print(f"  WARN: invalid JSON for {key}: {e}, skipping")
            continue

        if not isinstance(info, dict):
            print(f"  WARN: value is not a dict for {key}, skipping")
            continue

        subscribe = info.get("subscribe", 0)
        unionid = info.get("unionid", "")
        subscribe_time = info.get("subscribe_time", 0)
        language = info.get("language", "")
        remark = info.get("remark", "")
        groupid = info.get("groupid", 0)
        tagid_list = json.dumps(info.get("tagid_list", []), ensure_ascii=False)
        subscribe_scene = info.get("subscribe_scene", "")
        qr_scene = info.get("qr_scene", 0)
        qr_scene_str = info.get("qr_scene_str", "")

        def esc(val):
            if val is None:
                return "NULL"
            if isinstance(val, str):
                return "'" + val.replace("'", "''") + "'"
            if isinstance(val, bool):
                return "true" if val else "false"
            return str(val)

        sql = (
            f"INSERT INTO wx_follower_infos "
            f"(openid, app_id, subscribe, unionid, subscribe_time, language, remark, "
            f"groupid, tagid_list, subscribe_scene, qr_scene, qr_scene_str, refreshed_at) "
            f"VALUES ({esc(openid)}, {esc(app_id)}, {subscribe}, {esc(unionid)}, "
            f"{subscribe_time}, {esc(language)}, {esc(remark)}, {groupid}, "
            f"'{tagid_list.replace(chr(39), chr(39)+chr(39))}', "
            f"{esc(subscribe_scene)}, {qr_scene}, {esc(qr_scene_str)}, '{now}') "
            f"ON CONFLICT (openid, app_id) DO UPDATE SET "
            f"subscribe = EXCLUDED.subscribe, unionid = EXCLUDED.unionid, "
            f"subscribe_time = EXCLUDED.subscribe_time, language = EXCLUDED.language, "
            f"remark = EXCLUDED.remark, groupid = EXCLUDED.groupid, "
            f"tagid_list = EXCLUDED.tagid_list, subscribe_scene = EXCLUDED.subscribe_scene, "
            f"qr_scene = EXCLUDED.qr_scene, qr_scene_str = EXCLUDED.qr_scene_str, "
            f"refreshed_at = EXCLUDED.refreshed_at;"
        )
        psql_exec(db_url, sql)
        total_infos += 1

        if total_infos % 50 == 0:
            print(f"  ... {total_infos} wx_follower_infos processed")

    print(f"\nTotal wx_follower_infos migrated: {total_infos}")


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} DATABASE_URL", file=sys.stderr)
        sys.exit(1)

    db_url = sys.argv[1]

    print("=" * 60)
    print("Redis → PostgreSQL WeChat Follower Data Migration")
    print("=" * 60)

    # Verify Redis connectivity
    pong = redis_cli("PING")
    if pong != "PONG":
        print("ERROR: Cannot connect to Redis at localhost:6379", file=sys.stderr)
        sys.exit(1)
    print("Redis: OK")

    # Verify the wx_followers table exists
    check = psql_exec(db_url, "SELECT count(*) FROM wx_followers")
    if check is None:
        print("WARNING: wx_followers table not found. Run migration first.", file=sys.stderr)
        print("  cd htyuc_models && diesel migration run")
        sys.exit(1)
    print("PG: wx_followers table exists")

    print()

    if RUN_ALL_OPENIDS:
        print("--- Phase 1: wx_followers (openid lists) ---")
        migrate_openids(db_url)
        cnt = psql_exec(db_url, "SELECT count(*) FROM wx_followers")
        print(f"Verification: wx_followers has {cnt.strip()} rows")

    print()

    if RUN_OPENID_INFO:
        print("--- Phase 2: wx_follower_infos (detailed info) ---")
        migrate_openid_infos(db_url)
        cnt = psql_exec(db_url, "SELECT count(*) FROM wx_follower_infos")
        print(f"Verification: wx_follower_infos has {cnt.strip()} rows")

    print()
    print("Migration complete.")


if __name__ == "__main__":
    main()
