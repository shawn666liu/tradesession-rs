import datetime as dt
from pathlib import Path


from tradesessionpy import TradeSession, SessionMgr


def test1():
    hm930 = dt.time(9, 30)
    hm1140 = dt.time(11, 40)
    empty = TradeSession([])
    print(f"{empty}")
    full = TradeSession.new_full_session()
    print(f"{full}")
    in1 = full.in_session(hm930, True)
    out1 = full.in_session(hm1140, True)
    assert in1 is True
    assert out1 is False

    mgr = SessionMgr()
    file = Path(__file__).parent.parent / "tradesession-rs" / "tradesession.csv"
    mgr.reload_csv_file(str(file), merge=True)
    print(f"sessions count = {mgr.sessions_count}")
    for k, v in mgr.session_map().items():
        print(f"{k}:\n{v}\n")
    pass


test1()
