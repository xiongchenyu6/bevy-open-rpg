// Contextual chapter preparation and offline management, reached through settings.
export function createDownloadUI({worker, request, language}) {
  const dialog = document.createElement('dialog');
  dialog.id = 'downloads-dialog';
  dialog.setAttribute('aria-labelledby', 'downloads-title');
  dialog.innerHTML = `<h2 id="downloads-title"></h2><p id="downloads-summary" aria-live="polite"></p><progress id="downloads-meter" hidden></progress><div id="downloads-groups"></div><p id="downloads-note"></p><div class="downloads-actions"><button id="downloads-confirm" type="button"></button><button id="downloads-remove" type="button"></button><button id="downloads-close" type="button"></button></div>`;
  document.body.append(dialog);
  window.loveDownloadsOpen = false;
  dialog.addEventListener('close', () => { window.loveDownloadsOpen = false; });
  const title = dialog.querySelector('h2'), summary = dialog.querySelector('#downloads-summary');
  const meter = dialog.querySelector('progress'), groups = dialog.querySelector('#downloads-groups');
  const note = dialog.querySelector('#downloads-note'), confirm = dialog.querySelector('#downloads-confirm');
  const remove = dialog.querySelector('#downloads-remove'), close = dialog.querySelector('#downloads-close');
  const text = {
    en: {chapter: 'Prepare chapter', manage: 'Downloads & offline play', checking: 'Checking saved resources…',
      size: n => `${n} to download`, ready: 'These resources are saved and ready to play offline.',
      shared: 'Shared world resources are reused. Only this chapter’s cinematic art is downloaded; later chapters remain optional.',
      optional: 'Download everything for offline play. Saved files are reused; only missing content is fetched.',
      progress: (a,b) => `Saving resources · ${a} / ${b}`, error: 'Download incomplete. Check your connection and free storage, then retry. Saved files will be reused.',
      unavailable: 'Offline storage is unavailable. This expedition will load online.',
      online: 'Continue online', download: 'Download & play', all: 'Download all for offline play', retry: 'Retry download',
      pause: 'Pause download', close: 'Back', cancel: 'Cancel download', clear: 'Remove downloaded extras',
      protected: 'Resources in an open expedition cannot be removed. Close other game windows and return to the title screen first.',
      removed: 'Extra downloads removed. Your progress and launcher are safe.',
      core: 'Launcher & shared interface', common: 'Shared world resources', optionalGroup: 'Credits & supporting files',
      saved: 'Saved', missing: 'Not downloaded', used: n => `Saved content: ${n}`},
    'zh-CN': {chapter: '准备章节', manage: '下载与离线游玩', checking: '正在检查已保存的资源…',
      size: n => `需要下载 ${n}`, ready: '资源已保存，可以离线游玩。',
      shared: '共享世界资源只下载一次；本卷演出按需下载，未到达的章节无需提前下载。',
      optional: '可下载全部内容供离线游玩。已有文件会复用，只下载缺少的内容。',
      progress: (a,b) => `正在保存资源 · ${a} / ${b}`, error: '下载未完成。请检查网络和可用空间后重试，已保存文件会继续复用。',
      unavailable: '当前无法保存离线资源，本次远征将联网加载。',
      online: '联网继续', download: '下载并开始', all: '下载全部离线内容', retry: '重试下载', pause: '暂停下载', close: '返回', cancel: '取消下载',
      clear: '移除已下载的额外内容', protected: '正在使用的远征资源无法移除。请关闭其他游戏窗口，并先返回标题画面。',
      removed: '额外下载已移除，游戏进度和启动资源已保留。',
      core: '启动与公共界面', common: '共享世界资源', optionalGroup: '制作名单与附属文件', saved: '已保存', missing: '未下载', used: n => `已保存内容：${n}`},
  };
  const bytes = n => n < 1048576 ? `${Math.ceil(n / 1024)} KB` : `${(n / 1048576).toFixed(1)} MB`;
  const chapterGroups = stage => stage === 0 ? ['common'] : ['common', 'chapter' + (stage + 1)];
  let transition = false;
  let mode = 'manage', stage = 0, status = null, phase = 'checking', busy = false;
  let received = 0, total = 0, generation = 0, currentWorker;
  function render() {
    const t = text[language()] || text.en;
    title.textContent = t[mode === 'chapter' ? 'chapter' : 'manage'];
    summary.textContent = phase === 'progress' ? t.progress(bytes(received), bytes(total)) :
      ['error','protected','removed','unavailable'].includes(phase) ? t[phase] :
      !status ? t.checking : status.downloadBytes ? t.size(bytes(status.downloadBytes)) : t.ready;
    note.textContent = t[mode === 'chapter' ? 'shared' : 'optional'];
    confirm.textContent = phase === 'unavailable' ? t.online : phase === 'error' ? t.retry : t[mode === 'chapter' ? 'download' : 'all'];
    confirm.hidden = mode === 'manage' && (status?.ready === true || phase === 'unavailable');
    confirm.disabled = busy || phase === 'checking';
    close.textContent = t[mode === 'chapter' && transition ? 'pause' : busy ? 'cancel' : 'close'];
    remove.textContent = t.clear;
    remove.hidden = mode !== 'manage'; remove.disabled = busy || !status;
    meter.hidden = phase !== 'progress'; meter.max = Math.max(1,total); meter.value = received;
    groups.replaceChildren();
    if (mode === 'manage' && status) {
      for (const group of status.groups || []) {
        const row = document.createElement('p');
        row.textContent = `${t[group.id === 'optional' ? 'optionalGroup' : group.id] || (language() === 'zh-CN' ? '章节 ' : 'Chapter ') + group.id.replace('chapter', '')} · ${group.ready ? t.saved : t.missing} · ${bytes(group.totalBytes - group.downloadBytes)} / ${bytes(group.totalBytes)}`;
        groups.append(row);
      }
    }
  }
  async function rpc(type, onProgress, selected) {
    currentWorker = await worker();
    if (!currentWorker) throw new Error('OFFLINE_STORAGE_UNAVAILABLE');
    return request(currentWorker, type, onProgress, type === 'PREPARE' ? 120000 : 15000, 'client', selected);
  }
  let pinQueue = Promise.resolve();
  function pinGroups(selected) {
    pinQueue = pinQueue.catch(() => {}).then(() => rpc('PIN_GROUPS', null, selected));
    return pinQueue;
  }
  async function finishChapter(id) {
    await pinGroups(chapterGroups(stage));
    if (id !== generation) return;
    window.loveChapterDownload = {stage, status: 'ready'};
    dialog.close();
  }
  async function show(nextMode, nextStage = 0) {
    if (busy) return;
    const id = ++generation;
    mode = nextMode; stage = nextStage; status = null; phase = 'checking';
    if (mode === 'chapter') window.loveChapterDownload = {stage, status: 'downloading'};
    render(); if (!dialog.open) { window.loveDownloadsOpen = true; dialog.showModal(); }
    try {
      const result = await rpc('STATUS', null, mode === 'chapter' ? chapterGroups(stage) : ['all']);
      if (id !== generation) return;
      status = result;
      phase = 'ready';
      if (mode === 'chapter' && status.ready) await finishChapter(id);
    } catch (error) {
      if (id !== generation) return;
      console.warn('Resource status unavailable', error); phase = error.message === 'OFFLINE_STORAGE_UNAVAILABLE' ? 'unavailable' : 'error';
    }
    render();
  }
  confirm.addEventListener('click', async () => {
    if (busy) return;
    if (phase === 'unavailable' && mode === 'chapter') {
      window.loveChapterDownload = {stage, status: 'ready'}; dialog.close(); return;
    }
    const id = generation; busy = true; phase = 'progress'; received = 0; total = status?.downloadBytes || 0; render();
    try {
      const result = await rpc('PREPARE', data => {
        if (id !== generation) return;
        received = data.received; total = data.total; render();
      }, mode === 'chapter' ? chapterGroups(stage) : ['all']);
      if (id !== generation) return;
      status = result;
      phase = 'ready';
      if (mode === 'chapter') await finishChapter(id);
    } catch (error) {
      if (id !== generation) return;
      console.warn('Resources remain incomplete', error); phase = 'error';
    } finally { if (id === generation) { busy = false; render(); } }
  });
  async function back() {
    const wasBusy = busy;
    generation++;
    busy = false;
    if (mode === 'chapter' && transition) {
      if (wasBusy && currentWorker) await request(currentWorker, 'CANCEL').catch(() => {});
      phase = 'error'; busy = false; render(); return;
    }
    if (mode === 'chapter') {
      window.loveChapterDownload = {stage, status: 'cancelled'};
      void pinGroups([]).catch(() => {});
    }
    dialog.close();
    if (wasBusy && currentWorker) await request(currentWorker, 'CANCEL').catch(() => {});
  }
  close.addEventListener('click', () => { void back(); });
  dialog.addEventListener('cancel', event => { event.preventDefault(); void back(); });
  remove.addEventListener('click', async () => {
    busy = true; render();
    try {
      await rpc('REMOVE_GROUPS', null, status.groups.filter(group => group.id !== 'core').map(group => group.id));
      status = await rpc('STATUS', null, ['all']); phase = 'removed';
    } catch { phase = 'protected'; }
    finally { busy = false; render(); }
  });
  window.lovePrepareChapter = (stage, betweenChapters = false) => { transition = betweenChapters; void show('chapter', stage); };
  window.loveOpenDownloads = () => { void show('manage'); };
  window.loveReleaseChapter = () => pinGroups([]).catch(() => {});
  return {render};
}
