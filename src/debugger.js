#!/usr/bin/env node
/**
 * Aura Interactive Step Debugger (aurac debug --step / aurac step)
 * 
 * Provides an interactive CLI debugger for Aura programs:
 * - Line-by-line stepping mapped directly to original .aura files
 * - Automatically skips compiler-generated runtime helpers and Node internals
 * - Breakpoints, variable inspection, expressions evaluation, and call stack backtrace
 */

const fs = require('fs');
const path = require('path');
const http = require('http');
const readline = require('readline');
const { spawn } = require('child_process');

// ANSI Colors and Styles
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const DIM = '\x1b[2m';
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const MAGENTA = '\x1b[35m';
const CYAN = '\x1b[36m';

function decodeVLQ(str) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  const charMap = {};
  for (let i = 0; i < chars.length; i++) charMap[chars[i]] = i;
  const values = [];
  let value = 0;
  let shift = 0;
  for (let i = 0; i < str.length; i++) {
    const digit = charMap[str[i]];
    if (digit === undefined) continue;
    const continuation = digit & 32;
    value += (digit & 31) << shift;
    shift += 5;
    if (!continuation) {
      const isNegative = value & 1;
      const parsed = value >> 1;
      values.push(isNegative ? -parsed : parsed);
      value = 0;
      shift = 0;
    }
  }
  return values;
}

class AuraStepDebugger {
  constructor(auraPath, jsPath, mapPath, port) {
    this.auraPath = path.resolve(auraPath);
    this.jsPath = path.resolve(jsPath);
    this.mapPath = path.resolve(mapPath);
    this.port = port || 9229;

    this.auraLines = [];
    this.genLineToAura = new Map();
    this.auraLineToGen = new Map();
    this.activeBreakpoints = new Map(); // auraLine -> bpId

    this.childProc = null;
    this.ws = null;
    this.msgId = 1;
    this.pendingCallbacks = new Map();

    this.currentCallFrames = [];
    this.currentAuraLine = -1;
    this.steppingMode = 'init'; // 'init' | 'stepOver' | 'stepInto' | 'stepOut' | 'idle'
    this.lastPausedAuraLine = -1;
    this.lastCommand = 'n';

    this.rl = null;
    this.hasHitInitialBreak = false;
  }

  loadSourceAndMap() {
    try {
      const auraSrc = fs.readFileSync(this.auraPath, 'utf8');
      this.auraLines = auraSrc.split(/\r?\n/);
    } catch (e) {
      console.error(`${RED}Error reading Aura source file '${this.auraPath}': ${e.message}${RESET}`);
      process.exit(1);
    }

    try {
      const mapRaw = fs.readFileSync(this.mapPath, 'utf8');
      const map = JSON.parse(mapRaw);
      const lines = map.mappings.split(';');

      let prevSrcLine = 0;
      let prevSrcCol = 0;
      let prevSrcFile = 0;

      for (let genLine = 0; genLine < lines.length; genLine++) {
        const line = lines[genLine];
        if (!line) continue;
        const segments = line.split(',');
        let prevGenCol = 0;
        for (const seg of segments) {
          if (!seg) continue;
          const fields = decodeVLQ(seg);
          prevGenCol += fields[0];
          if (fields.length > 1) prevSrcFile += fields[1];
          if (fields.length > 2) prevSrcLine += fields[2];
          if (fields.length > 3) prevSrcCol += fields[3];

          if (!this.genLineToAura.has(genLine)) {
            this.genLineToAura.set(genLine, {
              auraLine: prevSrcLine + 1, // 1-indexed
              auraCol: prevSrcCol + 1,
              sourceFile: map.sources[prevSrcFile] || path.basename(this.auraPath)
            });
          }

          if (!this.auraLineToGen.has(prevSrcLine + 1)) {
            this.auraLineToGen.set(prevSrcLine + 1, genLine);
          }
        }
      }
    } catch (e) {
      console.error(`${RED}Error loading sourcemap '${this.mapPath}': ${e.message}${RESET}`);
      process.exit(1);
    }
  }

  findInitialGenBreakLine() {
    // Prefer the first executable statement inside `main()`
    for (let i = 0; i < this.auraLines.length; i++) {
      const line = this.auraLines[i].trim();
      if ((line.startsWith('fn main') || line.startsWith('export fn main') || line.startsWith('async fn main') || line.startsWith('export async fn main'))) {
        // Find first mapped line after function declaration
        for (let j = i + 1; j < Math.min(i + 15, this.auraLines.length); j++) {
          const genLine = this.auraLineToGen.get(j + 1);
          if (genLine !== undefined) {
            return genLine;
          }
        }
      }
    }

    // Fallback: the very first mapped Aura line
    let minGenLine = Infinity;
    for (const genLine of this.genLineToAura.keys()) {
      if (genLine < minGenLine) minGenLine = genLine;
    }
    return minGenLine !== Infinity ? minGenLine : 0;
  }

  send(method, params = {}) {
    return new Promise((resolve) => {
      const id = this.msgId++;
      this.pendingCallbacks.set(id, resolve);
      const payload = JSON.stringify({ id, method, params });
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(payload);
      }
    });
  }

  async start() {
    this.loadSourceAndMap();

    console.log(`\n${BOLD}${CYAN}⚡ Aura Interactive Step Debugger${RESET}`);
    console.log(`${DIM}Target file: ${this.auraPath}${RESET}`);
    console.log(`${DIM}Type 'h' for command help, ENTER to step over.${RESET}\n`);

    // Spawn child process with Node inspect-brk
    this.childProc = spawn('node', [
      '--enable-source-maps',
      `--inspect-brk=127.0.0.1:${this.port}`,
      this.jsPath
    ], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    this.childProc.stdout.on('data', (d) => {
      process.stdout.write(d.toString());
    });

    this.childProc.stderr.on('data', (d) => {
      const s = d.toString();
      if (!s.includes('Debugger listening on ws://') && !s.includes('For help, see: https://nodejs.org')) {
        process.stderr.write(s);
      }
    });

    this.childProc.on('exit', (code) => {
      if (this.rl) this.rl.close();
      console.log(`\n${DIM}— Program finished with exit code ${code ?? 0} —${RESET}`);
      process.exit(code ?? 0);
    });

    const wsUrl = await this.waitForWsUrl(this.port, 3000);
    if (!wsUrl) {
      console.error(`${RED}Failed to connect to Node.js inspector on port ${this.port}.${RESET}`);
      this.cleanup();
      process.exit(1);
    }

    this.connectWs(wsUrl);
  }

  waitForWsUrl(port, timeoutMs) {
    const start = Date.now();
    return new Promise((resolve) => {
      const check = () => {
        const req = http.get(`http://127.0.0.1:${port}/json/list`, (res) => {
          let data = '';
          res.on('data', (chunk) => (data += chunk));
          res.on('end', () => {
            try {
              const list = JSON.parse(data);
              if (list && list.length > 0 && list[0].webSocketDebuggerUrl) {
                return resolve(list[0].webSocketDebuggerUrl);
              }
            } catch (_) {}
            retry();
          });
        });
        req.on('error', () => retry());

        function retry() {
          if (Date.now() - start > timeoutMs) {
            resolve(null);
          } else {
            setTimeout(check, 100);
          }
        }
      };
      check();
    });
  }

  connectWs(wsUrl) {
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = async () => {
      await this.send('Debugger.enable');
      await this.send('Runtime.enable');
      await this.send('Debugger.setBlackboxPatterns', {
        patterns: ['node:.*', '<node_internals>.*']
      });

      // Set initial entry breakpoint
      const initGenLine = this.findInitialGenBreakLine();
      const escapedPath = path.basename(this.jsPath);
      await this.send('Debugger.setBreakpointByUrl', {
        lineNumber: initGenLine,
        urlRegex: `.*${escapedPath}`
      });

      // Resume from inspect-brk wait
      await this.send('Runtime.runIfWaitingForDebugger');
    };

    this.ws.onmessage = async (evt) => {
      try {
        const msg = JSON.parse(evt.data);
        if (msg.id && this.pendingCallbacks.has(msg.id)) {
          const cb = this.pendingCallbacks.get(msg.id);
          this.pendingCallbacks.delete(msg.id);
          cb(msg.result);
          return;
        }

        if (msg.method === 'Debugger.paused') {
          await this.handlePaused(msg.params);
        } else if (msg.method === 'Debugger.resumed') {
          // Resumed
        } else if (msg.method === 'Runtime.executionContextDestroyed') {
          if (this.rl) this.rl.close();
          console.log(`\n${DIM}— Program finished successfully —${RESET}`);
          this.cleanup();
          process.exit(0);
        }
      } catch (err) {
        console.error('WS Error:', err);
      }
    };

    this.ws.onerror = (e) => {
      console.error(`${RED}WebSocket error:${RESET}`, e.message || e);
    };
  }

  async handlePaused(params) {
    this.currentCallFrames = params.callFrames || [];
    if (this.currentCallFrames.length === 0) {
      await this.send('Debugger.stepOver');
      return;
    }

    // On initial inspect-brk stop, resume so we reach the first breakpoint in main
    if (!this.hasHitInitialBreak && params.reason === 'Break on start') {
      this.hasHitInitialBreak = true;
      await this.send('Debugger.resume');
      return;
    }

    const topFrame = this.currentCallFrames[0];
    const jsLine = topFrame.location.lineNumber;
    const url = topFrame.url || '';

    // If in node internals or unmapped runtime helpers, keep stepping
    const mapping = this.genLineToAura.get(jsLine);
    if (!mapping || url.startsWith('node:')) {
      if (this.steppingMode === 'stepInto') {
        await this.send('Debugger.stepInto');
      } else {
        await this.send('Debugger.stepOver');
      }
      return;
    }

    const auraLine = mapping.auraLine;

    // If stepping and still on the same Aura line (e.g. multi-step expression), continue stepping
    if ((this.steppingMode === 'stepOver' || this.steppingMode === 'stepInto') && auraLine === this.lastPausedAuraLine) {
      if (this.steppingMode === 'stepInto') {
        await this.send('Debugger.stepInto');
      } else {
        await this.send('Debugger.stepOver');
      }
      return;
    }

    // Arrived at a distinct line in the Aura source file
    this.steppingMode = 'idle';
    this.currentAuraLine = auraLine;
    this.lastPausedAuraLine = auraLine;

    this.displayLocation(auraLine, topFrame.functionName);
    this.promptUser();
  }

  displayLocation(lineNum, fnName) {
    const basename = path.basename(this.auraPath);
    const fnDisplay = fnName ? ` in ${CYAN}${fnName}()${RESET}` : '';
    console.log(`\n${GREEN}📍 [${basename}:${lineNum}]${RESET}${fnDisplay}`);

    const start = Math.max(0, lineNum - 3);
    const end = Math.min(this.auraLines.length, lineNum + 2);

    for (let i = start; i < end; i++) {
      const curLine = i + 1;
      const isCurrent = curLine === lineNum;
      const hasBp = this.activeBreakpoints.has(curLine);

      const bpMarker = hasBp ? `${RED}●${RESET}` : ' ';
      const pointer = isCurrent ? `${YELLOW}➜${RESET}` : ' ';
      const lineStr = String(curLine).padStart(4, ' ');
      const code = this.auraLines[i] || '';

      if (isCurrent) {
        console.log(`${bpMarker} ${pointer} ${BOLD}${lineStr} | ${code}${RESET}`);
      } else {
        console.log(`${bpMarker} ${pointer} ${DIM}${lineStr} | ${code}${RESET}`);
      }
    }
  }

  promptUser() {
    if (!this.rl) {
      this.rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
        prompt: `${BOLD}${CYAN}(aura-dbg)${RESET} `
      });

      this.rl.on('line', (line) => this.onCommand(line.trim()));
      this.rl.on('close', () => this.cleanup());
    }

    this.rl.prompt();
  }

  async onCommand(cmd) {
    if (!cmd) {
      // Repeat last stepping command on empty ENTER
      cmd = this.lastCommand || 'n';
    }

    const parts = cmd.split(/\s+/);
    const action = parts[0].toLowerCase();
    const arg = parts.slice(1).join(' ');

    switch (action) {
      case 'n':
      case 'next':
      case 'step': {
        this.lastCommand = 'n';
        this.steppingMode = 'stepOver';
        await this.send('Debugger.stepOver');
        break;
      }

      case 's':
      case 'into': {
        this.lastCommand = 's';
        this.steppingMode = 'stepInto';
        await this.send('Debugger.stepInto');
        break;
      }

      case 'o':
      case 'out':
      case 'finish': {
        this.lastCommand = 'o';
        this.steppingMode = 'stepOut';
        await this.send('Debugger.stepOut');
        break;
      }

      case 'c':
      case 'continue': {
        this.steppingMode = 'idle';
        await this.send('Debugger.resume');
        break;
      }

      case 'b':
      case 'break': {
        const lineNum = parseInt(arg, 10);
        if (isNaN(lineNum)) {
          console.log(`${YELLOW}Usage: b <line_number>${RESET}`);
          this.rl.prompt();
          return;
        }

        if (this.activeBreakpoints.has(lineNum)) {
          const bpId = this.activeBreakpoints.get(lineNum);
          await this.send('Debugger.removeBreakpoint', { breakpointId: bpId });
          this.activeBreakpoints.delete(lineNum);
          console.log(`${YELLOW}Breakpoint removed at line ${lineNum}${RESET}`);
        } else {
          const targetGenLine = this.auraLineToGen.get(lineNum);
          if (targetGenLine === undefined) {
            console.log(`${RED}No executable Aura code found at line ${lineNum}${RESET}`);
            this.rl.prompt();
            return;
          }

          const escapedPath = path.basename(this.jsPath);
          const res = await this.send('Debugger.setBreakpointByUrl', {
            lineNumber: targetGenLine,
            urlRegex: `.*${escapedPath}`,
            columnNumber: 0
          });

          if (res && res.breakpointId) {
            this.activeBreakpoints.set(lineNum, res.breakpointId);
            console.log(`${GREEN}Breakpoint set at line ${lineNum} (JS line ${targetGenLine})${RESET}`);
          } else {
            console.log(`${RED}Failed to set breakpoint at line ${lineNum}${RESET}`);
          }
        }
        this.rl.prompt();
        break;
      }

      case 'p':
      case 'print':
      case 'eval': {
        if (!arg) {
          console.log(`${YELLOW}Usage: p <expression>${RESET}`);
          this.rl.prompt();
          return;
        }

        if (this.currentCallFrames.length === 0) {
          console.log(`${RED}No active call frame.${RESET}`);
          this.rl.prompt();
          return;
        }

        const topFrame = this.currentCallFrames[0];
        const res = await this.send('Debugger.evaluateOnCallFrame', {
          callFrameId: topFrame.callFrameId,
          expression: arg,
          returnByValue: true
        });

        if (res && res.result) {
          const val = res.result;
          if (val.type === 'string') {
            console.log(`${GREEN}"${val.value}"${RESET}`);
          } else if (val.type === 'number' || val.type === 'boolean') {
            console.log(`${CYAN}${val.value}${RESET}`);
          } else if (val.type === 'undefined') {
            console.log(`${DIM}undefined${RESET}`);
          } else if (val.value !== undefined) {
            console.dir(val.value, { colors: true, depth: 3 });
          } else {
            console.log(`${val.description || val.className || val.type}`);
          }
        } else if (res && res.exceptionDetails) {
          console.log(`${RED}Evaluation Error: ${res.exceptionDetails.text || 'unknown'}${RESET}`);
        }
        this.rl.prompt();
        break;
      }

      case 'vars':
      case 'locals': {
        if (this.currentCallFrames.length === 0) {
          console.log(`${RED}No active call frame.${RESET}`);
          this.rl.prompt();
          return;
        }

        const topFrame = this.currentCallFrames[0];
        const localScope = topFrame.scopeChain.find((s) => s.type === 'local' || s.type === 'block');

        if (!localScope || !localScope.object || !localScope.object.objectId) {
          console.log(`${DIM}No local variables in current scope.${RESET}`);
          this.rl.prompt();
          return;
        }

        const props = await this.send('Runtime.getProperties', {
          objectId: localScope.object.objectId,
          ownProperties: true
        });

        if (props && props.result) {
          console.log(`${BOLD}Local Variables:${RESET}`);
          let count = 0;
          for (const p of props.result) {
            if (p.name.startsWith('__aura') || p.name === 'this') continue;
            count++;
            const v = p.value ? (p.value.value !== undefined ? JSON.stringify(p.value.value) : p.value.description) : 'undefined';
            console.log(`  ${CYAN}${p.name}${RESET} = ${GREEN}${v}${RESET}`);
          }
          if (count === 0) {
            console.log(`  ${DIM}(no user variables defined yet in this scope)${RESET}`);
          }
        }
        this.rl.prompt();
        break;
      }

      case 'bt':
      case 'backtrace':
      case 'stack': {
        console.log(`${BOLD}Call Stack:${RESET}`);
        for (let i = 0; i < this.currentCallFrames.length; i++) {
          const frame = this.currentCallFrames[i];
          const jsL = frame.location.lineNumber;
          const map = this.genLineToAura.get(jsL);
          const fn = frame.functionName || '<anonymous>';
          if (map) {
            console.log(`  #${i} ${CYAN}${fn}()${RESET} at ${map.sourceFile}:${map.auraLine}`);
          } else {
            console.log(`  #${i} ${DIM}${fn}() [runtime]${RESET}`);
          }
        }
        this.rl.prompt();
        break;
      }

      case 'l':
      case 'list': {
        const line = this.currentAuraLine > 0 ? this.currentAuraLine : 1;
        this.displayLocation(line, this.currentCallFrames[0]?.functionName);
        this.rl.prompt();
        break;
      }

      case 'h':
      case 'help': {
        console.log(`
${BOLD}Aura Interactive Step Debugger Commands:${RESET}
  ${CYAN}n${RESET}, ${CYAN}next${RESET}, ${CYAN}step${RESET}    Step over to next line in .aura source (default: press ENTER)
  ${CYAN}s${RESET}, ${CYAN}into${RESET}            Step into function call
  ${CYAN}o${RESET}, ${CYAN}out${RESET}, ${CYAN}finish${RESET}     Step out of current function
  ${CYAN}c${RESET}, ${CYAN}continue${RESET}        Continue execution until next breakpoint or exit
  ${CYAN}b <line>${RESET}         Toggle breakpoint at specified .aura line number
  ${CYAN}p <expr>${RESET}         Evaluate and print expression/variable on active frame
  ${CYAN}vars${RESET}, ${CYAN}locals${RESET}       Print all local variables in current scope
  ${CYAN}bt${RESET}, ${CYAN}backtrace${RESET}     Show mapped call stack backtrace
  ${CYAN}l${RESET}, ${CYAN}list${RESET}           Show surrounding Aura source code
  ${CYAN}h${RESET}, ${CYAN}help${RESET}           Show this help menu
  ${CYAN}q${RESET}, ${CYAN}quit${RESET}           Exit debugger
`);
        this.rl.prompt();
        break;
      }

      case 'q':
      case 'quit':
      case 'exit': {
        this.cleanup();
        process.exit(0);
        break;
      }

      default:
        console.log(`${YELLOW}Unknown command '${action}'. Type 'h' or 'help' for available commands.${RESET}`);
        this.rl.prompt();
        break;
    }
  }

  cleanup() {
    if (this.ws) {
      try { this.ws.close(); } catch (_) {}
    }
    if (this.childProc) {
      try { this.childProc.kill('SIGKILL'); } catch (_) {}
    }
  }
}

// CLI Execution Entrypoint
if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.length < 3) {
    console.log('Usage: node debugger.js <aura_file> <target_js> <map_file> [port]');
    process.exit(1);
  }

  const [auraFile, jsFile, mapFile, port] = args;
  const dbg = new AuraStepDebugger(auraFile, jsFile, mapFile, port ? parseInt(port, 10) : 9229);
  dbg.start().catch((err) => {
    console.error(`${RED}Fatal debugger error:${RESET}`, err);
    process.exit(1);
  });
}

module.exports = { AuraStepDebugger };
