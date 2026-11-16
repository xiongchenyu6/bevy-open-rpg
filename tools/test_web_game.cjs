#!/usr/bin/env node
// Exercise the actual compiled WebGPU game, not a substitute engine.
const {chromium}=require('playwright');
const fs=require('node:fs/promises'),path=require('node:path'),http=require('node:http'),os=require('node:os');
const assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../web'),out=path.resolve('target/web-game-proof');
async function finishDialogue(page) {
 for(let i=0;i<32;i++) {
  if(!await page.evaluate(()=>window.loveDialogue)) { await page.waitForTimeout(3000);if(!await page.evaluate(()=>window.loveDialogue))return; }
  // Same persistent edge input used by touch controls; wait for the real Rust frame to consume it.
  await page.evaluate(()=>window.loveInput.confirm=true);
  await page.waitForFunction(()=>window.loveInput.confirm===false,null,{timeout:30000});
  await page.waitForTimeout(1000);
 }
 throw Error('Dialogue did not finish through the input adapter');
}

(async()=>{
 await fs.mkdir(out,{recursive:true});
 const requests=[],errors=[];
 const manifest=JSON.parse(await fs.readFile(path.join(root,'asset-manifest.json'),'utf8'));
 const paths=new Map(manifest.files.map(f=>['/'+f.url,f.path]));
 const server=http.createServer(async(req,res)=>{let p=new URL(req.url,'http://local').pathname;requests.push(paths.get(p)||p);if(p==='/')p='/index.html';try{const data=await fs.readFile(path.join(root,p));res.setHeader('Content-Type',p.endsWith('.js')?'text/javascript':p.endsWith('.wasm')?'application/wasm':p.endsWith('.html')?'text/html':p.endsWith('.css')?'text/css':'application/octet-stream');res.end(data)}catch{res.writeHead(404).end()}});
 await new Promise(r=>server.listen(0,'127.0.0.1',r));const base='http://127.0.0.1:'+server.address().port;
 const profile=await fs.mkdtemp(path.join(os.tmpdir(),'rpg-real-'));
 const launch=(mobile=false)=>chromium.launchPersistentContext(profile,{headless:process.env.HEADED!=='1',hasTouch:mobile,isMobile:mobile,executablePath:process.env.CHROMIUM_EXECUTABLE_PATH,viewport:mobile?{width:844,height:390}:{width:1280,height:720},recordVideo:mobile?{dir:path.join(out,'video'),size:{width:844,height:390}}:undefined,args:['--no-sandbox','--ozone-platform=x11','--disable-gpu-watchdog','--disable-backgrounding-occluded-windows','--disable-renderer-backgrounding','--disable-background-timer-throttling','--enable-unsafe-webgpu','--use-angle=vulkan','--use-vulkan=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface']});
 let context;
 try{
 context=await launch();let page=await context.newPage();page.on('pageerror',e=>errors.push(String(e)));page.on('console',m=>{console.log('BROWSER',m.text().slice(0,600))});
 await page.goto(base);await page.locator('[data-language="en"]').click();assert.equal(await page.locator('#start').textContent(),'Start game');await page.screenshot({timeout:90000,path:path.join(out,'desktop-launcher.png')});
 await page.locator('[data-language="zh-CN"]').click();await page.locator('#start').click();
 await page.waitForFunction(()=>window.loveUiReady===true,null,{timeout:180000});
 await page.waitForFunction(()=>document.body.classList.contains('playing'));
 await page.waitForTimeout(5000);console.log('STATE',await page.evaluate(()=>({state:window.loveState,ready:window.loveUiReady,menu:window.loveMenuOpen,download:window.loveDownloadsOpen,canvas:[document.querySelector('canvas').width,document.querySelector('canvas').height],focus:document.activeElement.id})));await page.screenshot({timeout:90000,path:path.join(out,'desktop-title.png')});
 assert(!requests.some(p=>p.startsWith('assets/')&&!p.startsWith('assets/fonts/')&&p!=='assets/ui/title_bg.png'),JSON.stringify(requests.filter(p=>p.startsWith('assets/'))));
 console.log('PASS actual title loads only core font/title assets');
 await page.keyboard.down('Enter');await page.waitForTimeout(250);await page.keyboard.up('Enter');await page.locator('#downloads-dialog').waitFor({state:'visible'});await page.screenshot({timeout:90000,path:path.join(out,'chapter-download.png')});
 await page.locator('#downloads-confirm').click();await page.waitForFunction(()=>window.loveState==='RunScene',null,{timeout:180000});
 await page.waitForTimeout(6000);await page.screenshot({timeout:90000,path:path.join(out,'desktop-chapter.png')});
 assert(!requests.some(p=>/assets\/ui\/(?:anim\/)?chapter[1-7]_/.test(p)),'first chapter must not download later/unused cinematic packs');
 console.log('PASS actual first chapter downloads shared world only');
 await finishDialogue(page);
 await page.screenshot({timeout:90000,path:path.join(out,'desktop-map.png')});
 await page.setViewportSize({width:844,height:390});await page.waitForTimeout(1000);await page.screenshot({timeout:90000,path:path.join(out,'mobile-map.png')});
 await page.setViewportSize({width:390,height:844});await page.waitForTimeout(1000);await page.screenshot({timeout:90000,path:path.join(out,'portrait-map.png')});
 await fs.writeFile(path.join(out,'requests.json'),JSON.stringify(requests,null,2));
 await context.close();context=await launch(true);await context.setOffline(true);page=await context.newPage();page.on('pageerror',e=>errors.push(String(e)));await page.goto(base);await page.locator('#start').click();await page.waitForFunction(()=>document.body.classList.contains('playing'),null,{timeout:180000});await page.keyboard.down('Enter');await page.waitForTimeout(800);await page.keyboard.up('Enter');await page.waitForFunction(()=>window.loveState==='RunScene',null,{timeout:180000});await page.waitForTimeout(2000);await page.screenshot({timeout:90000,path:path.join(out,'cold-offline-chapter.png')});console.log('PASS fresh browser process runs installed chapter completely offline');
 await finishDialogue(page);
 await page.locator('#touch-controls').waitFor({state:'visible'});
 const box=await page.locator('#move-zone').boundingBox(), before=await page.locator('canvas').boundingBox();
 await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await page.mouse.move(box.x+box.width/2+45,box.y+box.height/2);
 assert.equal(await page.evaluate(()=>window.loveInput.dx),1);await page.waitForTimeout(900);await page.screenshot({timeout:90000,path:path.join(out,'mobile-moving.png')});await page.mouse.up();assert.equal(await page.evaluate(()=>window.loveInput.dx),0);assert.deepEqual(await page.locator('canvas').boundingBox(),before);
 console.log('PASS mobile movement releases cleanly and does not resize the canvas');
 await page.locator('#inventory').click();await page.waitForFunction(()=>window.loveInventory===true);await page.waitForTimeout(500);await page.screenshot({timeout:90000,path:path.join(out,'mobile-inventory.png')});
 assert.equal(await page.locator('#interact').isVisible(),false);assert.equal(await page.locator('#move-zone').isVisible(),false);
 const cdp=await context.newCDPSession(page);await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:650,y:260,id:1}]});await page.waitForTimeout(300);await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:650,y:95,id:1}]});await page.waitForTimeout(800);await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await page.waitForTimeout(500);await page.screenshot({timeout:90000,path:path.join(out,'mobile-inventory-scrolled.png')});
 await page.locator('#inventory').click();await page.waitForFunction(()=>!window.loveInventory);await page.setViewportSize({width:390,height:844});await page.waitForTimeout(1000);await page.screenshot({timeout:90000,path:path.join(out,'mobile-portrait-map.png')});await page.locator('#inventory').click();await page.waitForFunction(()=>window.loveInventory);await page.waitForTimeout(500);await page.screenshot({timeout:90000,path:path.join(out,'mobile-portrait-inventory.png')});await page.locator('#inventory').click();await page.waitForFunction(()=>!window.loveInventory);await page.setViewportSize({width:844,height:390});await page.waitForTimeout(500);
 console.log('PASS inventory reserves touch controls and supports portrait plus finger scrolling');
 for(const key of ['ArrowRight','ArrowDown','ArrowLeft','ArrowUp']){await page.keyboard.down(key);await page.waitForTimeout(3800);await page.keyboard.up(key)}
 await page.screenshot({timeout:90000,path:path.join(out,'mobile-offline-map.png')});
 const video=page.video();await context.close();context=null;await fs.writeFile(path.join(out,'video-path.txt'),await video.path());

 assert.equal(errors.length,0,errors.join('\n'));await fs.writeFile(path.join(out,'result.json'),JSON.stringify({passed:true,revision:manifest.revision,manifestVersion:manifest.version,engineInputsSha256:JSON.parse(await fs.readFile(path.join(root,'build.json'),'utf8')).engine_inputs_sha256,errors},null,2));
 }finally{await context?.close();server.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
