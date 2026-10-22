// 線上房間的 WebRTC 連線（P2P）：開房者的瀏覽器就是伺服器，其他人直接連過去。
// 第一次連線時透過配對服務（workers/signal，Cloudflare）交換 offer / answer / ICE，之後牌局資料只走 WebRTC。
// 傳的訊息跟 WebSocket 伺服器一模一樣（JSON），所以 net.svelte.js 只要換掉傳輸層。

const ICE_SERVERS = [{ urls: ['stun:stun.cloudflare.com:3478', 'stun:stun.l.google.com:19302'] }];
const CONNECT_TIMEOUT = 20000;

export const p2pSupported = typeof RTCPeerConnection !== 'undefined' && typeof WebSocket !== 'undefined';

function signalSocket(base, path) {
  const u = new URL(base);
  return new WebSocket(`${u.protocol === 'https:' ? 'wss' : 'ws'}://${u.host}${path}`);
}

/**
 * 房主：在配對服務登記房號，等別人連進來。每個連進來的人給一個數字 id（從 1 開始；0 是房主自己）。
 * onTaken() 表示房號被別人用了，要換一個；onSignalDown(true/false) 回報配對服務斷線與否（已連上的人不受影響）。
 */
export class P2PHost {
  constructor({ signal, code, key, onPeerOpen, onPeerMessage, onPeerClose, onTaken, onSignalDown }) {
    Object.assign(this, { signal, code, key, onPeerOpen, onPeerMessage, onPeerClose, onTaken, onSignalDown });
    this.peers = new Map(); // 配對服務給的客人 id → { pc, dc, num }
    this.nextNum = 1;
    this.stopped = false;
    this.ws = null;
    this.retry = null;
  }

  start() {
    if (this.stopped) return;
    const ws = signalSocket(this.signal, `/r/${this.code}/host?key=${encodeURIComponent(this.key)}`);
    this.ws = ws;
    ws.onopen = () => this.onSignalDown?.(false);
    ws.onmessage = (e) => {
      let m;
      try {
        m = JSON.parse(e.data);
      } catch {
        return;
      }
      if (m.type === 'error' && m.message === 'taken') {
        this.stopped = true;
        this.onTaken?.();
      } else if (m.type === 'signal') {
        this.handleSignal(m.from, m.data);
      } else if (m.type === 'leave') {
        // 還沒連上就離開的人（例如關掉分頁）
        const p = this.peers.get(m.id);
        if (p && p.dc?.readyState !== 'open') this.dropPeer(m.id);
      }
    };
    ws.onclose = () => {
      if (this.ws !== ws || this.stopped) return;
      this.onSignalDown?.(true);
      this.retry = setTimeout(() => this.start(), 3000);
    };
  }

  async handleSignal(id, data) {
    if (!data) return;
    let p = this.peers.get(id);
    if (data.sdp?.type === 'offer') {
      if (p) this.dropPeer(id);
      const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
      p = { pc, dc: null, num: this.nextNum++, pending: [] };
      this.peers.set(id, p);
      pc.onicecandidate = (e) => {
        if (e.candidate) this.sendSignal(id, { candidate: e.candidate });
      };
      pc.ondatachannel = (e) => {
        const dc = e.channel;
        p.dc = dc;
        dc.onopen = () => this.onPeerOpen(p.num);
        dc.onmessage = (ev) => {
          let msg;
          try {
            msg = JSON.parse(ev.data);
          } catch {
            return;
          }
          this.onPeerMessage(p.num, msg);
        };
        dc.onclose = () => this.dropPeer(id);
      };
      pc.onconnectionstatechange = () => {
        if (pc.connectionState === 'failed' || pc.connectionState === 'closed') this.dropPeer(id);
      };
      await pc.setRemoteDescription(data.sdp);
      for (const c of p.pending) await pc.addIceCandidate(c).catch(() => {});
      p.pending = [];
      await pc.setLocalDescription(await pc.createAnswer());
      this.sendSignal(id, { sdp: pc.localDescription });
    } else if (data.candidate && p) {
      if (p.pc.remoteDescription) await p.pc.addIceCandidate(data.candidate).catch(() => {});
      else p.pending.push(data.candidate);
    }
  }

  sendSignal(to, data) {
    if (this.ws?.readyState === 1) this.ws.send(JSON.stringify({ to, data }));
  }

  dropPeer(id) {
    const p = this.peers.get(id);
    if (!p) return;
    this.peers.delete(id);
    const wasOpen = !!p.dc;
    try {
      p.dc?.close();
      p.pc.close();
    } catch {}
    if (wasOpen) this.onPeerClose(p.num);
  }

  /** 送訊息給第 num 號連線 */
  send(num, msg) {
    for (const p of this.peers.values()) {
      if (p.num === num && p.dc?.readyState === 'open') {
        p.dc.send(JSON.stringify(msg));
        return;
      }
    }
  }

  stop() {
    this.stopped = true;
    clearTimeout(this.retry);
    this.ws?.close();
    for (const id of [...this.peers.keys()]) this.dropPeer(id);
  }
}

/**
 * 加入的人：透過配對服務找到房主，建立 WebRTC 資料通道。連上之後就關掉配對服務的連線。
 * onOpen() 連上；onMessage(obj)；onClose() 斷線（可以重連）；onFatal(訊息) 不用再試了（例如房間不存在）。
 */
export class P2PGuest {
  constructor({ signal, code, onOpen, onMessage, onClose, onFatal }) {
    Object.assign(this, { signal, code, onOpen, onMessage, onClose, onFatal });
    this.closed = false;
  }

  connect() {
    const ws = signalSocket(this.signal, `/r/${this.code}/guest`);
    this.ws = ws;
    let opened = false;
    const fail = (fatal) => {
      if (this.closed || opened) return;
      this.cleanup();
      if (fatal) this.onFatal(fatal);
      else this.onClose();
    };
    this.timer = setTimeout(() => fail(), CONNECT_TIMEOUT);
    ws.onmessage = async (e) => {
      let m;
      try {
        m = JSON.parse(e.data);
      } catch {
        return;
      }
      if (m.type === 'error') {
        fail(m.message === 'no_host' ? `找不到房號 ${this.code}（房間可能已經關閉了）` : m.message);
      } else if (m.type === 'welcome') {
        const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
        this.pc = pc;
        const dc = pc.createDataChannel('game', { ordered: true });
        this.dc = dc;
        pc.onicecandidate = (ev) => {
          if (ev.candidate && ws.readyState === 1) ws.send(JSON.stringify({ data: { candidate: ev.candidate } }));
        };
        pc.onconnectionstatechange = () => {
          if (pc.connectionState === 'failed') {
            if (opened) this.lost();
            else fail();
          }
        };
        dc.onopen = () => {
          opened = true;
          clearTimeout(this.timer);
          ws.close(); // 連上之後就不需要配對服務了
          this.onOpen();
        };
        dc.onmessage = (ev) => {
          try {
            this.onMessage(JSON.parse(ev.data));
          } catch {}
        };
        dc.onclose = () => {
          if (opened) this.lost();
        };
        await pc.setLocalDescription(await pc.createOffer());
        ws.send(JSON.stringify({ data: { sdp: pc.localDescription } }));
      } else if (m.type === 'signal' && this.pc) {
        if (m.data?.sdp) await this.pc.setRemoteDescription(m.data.sdp);
        else if (m.data?.candidate) await this.pc.addIceCandidate(m.data.candidate).catch(() => {});
      }
    };
    ws.onerror = () => fail();
    ws.onclose = () => {
      if (!opened) fail();
    };
  }

  lost() {
    if (this.closed || this.lostOnce) return;
    this.lostOnce = true;
    this.cleanup();
    this.onClose();
  }

  get open() {
    return this.dc?.readyState === 'open';
  }

  send(msg) {
    if (this.open) this.dc.send(JSON.stringify(msg));
  }

  cleanup() {
    clearTimeout(this.timer);
    try {
      this.ws?.close();
      this.dc?.close();
      this.pc?.close();
    } catch {}
  }

  close() {
    this.closed = true;
    this.cleanup();
  }
}

/** 配對服務還在不在（介面用來決定要不要顯示「線上」房間） */
export async function signalAvailable(signal) {
  if (!signal || !p2pSupported) return false;
  try {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), 4000);
    const res = await fetch(`${signal}/health`, { signal: ctrl.signal });
    clearTimeout(t);
    return res.ok;
  } catch {
    return false;
  }
}
