import datetime as dt

from tradesessionpy import TradeSession, SessionMgr


def test1():
    hm930 = dt.time(9, 30)
    full = TradeSession([])
    full.in_session(hm930, True, False)
