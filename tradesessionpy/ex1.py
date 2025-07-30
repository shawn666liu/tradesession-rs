import datetime as dt
from pathlib import Path


from tradesessionpy import TradeSession, SessionMgr


def test1():
    hm930 = dt.time(9, 30)
    hm1140 = dt.time(11, 40)

    # 创建空Session
    empty = TradeSession([])
    print(f"empty session:\n{empty}")

    # 直接获取预置session
    builtin = TradeSession.new_bond_session()
    builtin = TradeSession.new_commodity_session_night()
    builtin = TradeSession.new_full_session()

    print(f"builtin session:\n{builtin}")
    in1 = builtin.in_session(hm930, True)
    out1 = builtin.in_session(hm1140, True)
    assert in1 is True
    assert out1 is False

    # 获取所有分钟开始时间
    minutes = builtin.minutes_list()
    print(f"minutes after shift:\n{minutes}\n")

    # 从分钟时间列表反向构建
    from_mins = TradeSession(minutes)
    print(f"from minutes:\n{from_mins}")

    # SessinMgr从文件加载
    mgr = SessionMgr()
    file = Path(__file__).parent.parent / "tradesession-rs" / "tradesession.csv"
    mgr.reload_csv_file(str(file), merge=True)
    print(f"sessions count = {mgr.sessions_count}")
    for k, v in mgr.session_map().items():
        print(f"{k}:\n{v}\n")
    pass

    # 获取某个特定品种的session
    try:
        ru = mgr.get_session("ru")
        print(f"ru:\n{ru}")

        Notfound = mgr.get_session("Ru")
    except Exception as ex:
        print(f"\nfailed to get session for `Ru`\n")

    # 创建一个CTP接口必须处于连接状态的Session
    ctp_should_connected = TradeSession([])
    ctp_should_connected.add_slice(8, 40, 15, 30)
    ctp_should_connected.add_slice(20, 40, 2, 31)
    ctp_should_connected.post_fix()
    print(f"ctp_should_connected:\n{ctp_should_connected}")


if __name__ == "__main__":
    test1()
