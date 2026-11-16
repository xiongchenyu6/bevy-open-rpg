import {setCacheLanguage,prepareGameCache,markGameStarted,consumeCacheResume} from './game-cache.js';
import {setInstallLanguage} from './pwa.js';
const $ = id => document.getElementById(id);
const copy = {
 'zh-CN': {title:'御剑行 · 轮回',tagline:'一局一世 · 随机路途 · 剧情织缘',start:'开始游戏',loading:'准备游戏资源…',ready:'点击开始；已下载的资源会保留，下次无需重复下载。',engine:'正在初始化画面…',error:'加载未完成，请检查网络后重试。',gpu:'当前浏览器未提供 WebGPU。请使用支持 WebGPU 的新版浏览器。',settings:'设置',downloads:'下载与离线游玩',back:'返回',gameLanguage:'游戏剧情与战斗界面目前使用简体中文。',rotate:'横屏游玩可以获得更大的地图和更清晰的选项。'},
 en:{title:'Love RPG · Rebirth',tagline:'One life · An unknown path · Stories that endure',start:'Start game',loading:'Preparing game resources…',ready:'Start when ready. Downloaded resources are kept for your next visit.',engine:'Preparing the first scene…',error:'Loading incomplete. Check your connection and retry.',gpu:'WebGPU is unavailable in this browser. Use an up-to-date browser with WebGPU support.',settings:'Settings',downloads:'Downloads & offline play',back:'Back',gameLanguage:'Story and in-game controls are currently in Simplified Chinese.',rotate:'Turn your phone sideways for a larger map and readable choices.'}
};
let language = new URL(location.href).searchParams.get('lang');
try { language ||= localStorage.getItem('love-rpg-language'); } catch {}
language = language === 'en' ? 'en' : 'zh-CN';
let phase='ready', busy=false, running=false, engineStarted=false;
function render() {
 const t=copy[language]; document.documentElement.lang=language; document.title=t.title;
 $('game-title').textContent=t.title; $('tagline').textContent=t.tagline; $('status').textContent=t[phase];
 $('start').textContent=busy?t.loading:t.start;
 for(const node of document.querySelectorAll('[data-copy]')) node.textContent=t[node.dataset.copy];
 for(const node of document.querySelectorAll('[data-language]')) node.setAttribute('aria-pressed',String(node.dataset.language===language));
 $('settings-entry').title=t.settings; $('settings-entry').setAttribute('aria-label',t.settings);
 setCacheLanguage(language); setInstallLanguage(language);
}
for(const button of document.querySelectorAll('[data-language]')) button.onclick=()=>{ language=button.dataset.language;try{localStorage.setItem('love-rpg-language',language)}catch{}render(); };
window.loveInput={dx:0,dy:0,confirm:false,cancel:false};
window.loveMenuOpen=false;
$('settings-entry').onclick=()=>{window.loveMenuOpen=true;$('settings-dialog').showModal()};
$('settings-close').onclick=()=>$('settings-dialog').close();
$('settings-dialog').onclose=()=>{window.loveMenuOpen=false};
$('manage-downloads').onclick=()=>window.loveOpenDownloads();
$('interact').onclick=()=>{window.loveInput.confirm=true};
$('inventory').onclick=()=>{window.loveInput.cancel=true};
const zone=$('move-zone'),stick=$('stick');let pointer,origin;
function release(){pointer=undefined;window.loveInput.dx=window.loveInput.dy=0;stick.hidden=true;}
zone.onpointerdown=e=>{if(pointer!==undefined)return;zone.dataset.used='true';e.preventDefault();pointer=e.pointerId;zone.setPointerCapture(pointer);origin={x:e.clientX,y:e.clientY};const r=zone.getBoundingClientRect();stick.style.left=(e.clientX-r.left)+'px';stick.style.top=(e.clientY-r.top)+'px';stick.hidden=false;};
zone.onpointermove=e=>{if(e.pointerId!==pointer)return;e.preventDefault();const dx=e.clientX-origin.x,dy=e.clientY-origin.y;const mag=Math.hypot(dx,dy),scale=Math.min(1,30/Math.max(1,mag));stick.firstElementChild.style.transform=`translate(${dx*scale}px,${dy*scale}px)`;window.loveInput.dx=Math.abs(dx)>Math.abs(dy)&&mag>10?Math.sign(dx):0;window.loveInput.dy=Math.abs(dy)>=Math.abs(dx)&&mag>10?Math.sign(dy):0;};
zone.onpointerup=zone.onpointercancel=zone.onlostpointercapture=release;
window.addEventListener('blur',release);document.addEventListener('visibilitychange',release);
const mobile=matchMedia('(pointer:coarse)').matches||navigator.maxTouchPoints>0;
window.loveMobile=mobile;
setInterval(()=>{
 const map=window.loveState==='RunScene'&&!window.loveDialogue&&!window.loveMenuOpen&&!window.loveDownloadsOpen;
 $('touch-controls').hidden=!running||!mobile||!map;
 if(!map)release();
 zone.hidden=!!window.loveInventory;$('interact').hidden=!!window.loveInventory;
 $('inventory').textContent=window.loveInventory?'返回':'行囊';
 if(window.loveInventory)release();
},100);
$('start').onclick=async()=>{
 if(busy)return;if(engineStarted){location.reload();return;}busy=true;$('start').disabled=true;phase='loading';render();
 try {
  const adapter=navigator.gpu&&await navigator.gpu.requestAdapter();
  if(!adapter){phase='gpu';return;}
  if(mobile){try{await document.documentElement.requestFullscreen?.();await screen.orientation?.lock?.('landscape')}catch{}}
  await prepareGameCache();
  const {default:init}=await import('./pkg/love_rpg.js');
  let bytes;
  if(typeof DecompressionStream==='function'){
   const response=await fetch('./pkg/love_rpg_bg.wasm.gz');if(!response.ok)throw Error('Engine download failed');
   bytes=await new Response(response.body.pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
  }
  phase='engine';render();
  engineStarted=true;
  try { await init(bytes?{module_or_path:bytes}:{module_or_path:'./pkg/love_rpg_bg.wasm'}); }
  catch(error){if(!String(error).includes('Using exceptions for control flow'))throw error;}
  await new Promise((resolve,reject)=>{const timer=setInterval(()=>{if(window.loveUiError){clearInterval(timer);reject(Error(window.loveUiError))}else if(window.loveUiReady){clearInterval(timer);resolve()}},80)});
  markGameStarted();running=true;document.body.classList.add('playing');$('loading').classList.add('hidden');$('settings-entry').hidden=false;$('bevy-canvas').focus();
 }catch(error){console.error(error);phase='error';}finally{busy=false;$('start').disabled=false;render();}
};
render();if(consumeCacheResume())$('start').click();
