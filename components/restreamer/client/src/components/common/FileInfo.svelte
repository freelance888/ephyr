<script lang='js'>
  import { mutation } from 'svelte-apollo';
  import { DownloadFile } from '../../../api/client.graphql';
  import { showError } from '../../utils/util';

  import Confirm from './Confirm.svelte';
  import { formatStreamInfo } from '../../utils/streamInfo.util';
  import StreamInfo from './StreamInfo.svelte';

  export let file;
  export let classList;

  $: fileDownloadProgress = getDownloadProgress(file);

  $: isDownloadError = file?.state === 'DOWNLOAD_ERROR';

  $: downloadErrorMessage = file?.error;

  $: fileName = file.name ? file.name : file.fileId;

  const downloadFileMutation = mutation(DownloadFile);
  async function downloadFile() {
    try {
      await downloadFileMutation({ variables: { fileId: file.fileId } });
    } catch (e) {
      showError(e.message);
    }
  }

  const getDownloadProgress = (f) => {
    let value =
      f?.downloadState &&
      f.downloadState.currentProgress !==
      f.downloadState.maxProgress
        ? (f.downloadState.currentProgress /
          f.downloadState.maxProgress) * 100
        : 0;

    return value < 0 || value >= 100 ? undefined : value;
  }

</script>

<template>
  <Confirm let:confirm>
    <div class="file-info-container uk-flex uk-flex-middle {classList}">
      <div class="uk-flex uk-flex-column">
        <div class="uk-flex uk-flex-middle">
          <span
            href="/"
            class="file-name uk-display-inline-block"
          >
            {fileName}
          </span>
          {#if isDownloadError}
            <span
              class="info-icon has-error"
              uk-icon="icon: info; ratio: 0.7"
              uk-tooltip={downloadErrorMessage}
            />
          {/if}
        </div>

        <div class="uk-flex uk-flex-middle">
          {#if fileDownloadProgress}
            <progress
              class="uk-progress"
              value={fileDownloadProgress}
              max="100"
            />
            <span class="download-percents"
            >{fileDownloadProgress.toFixed(0)}</span
            >%
          {/if}
        </div>
      </div>
      {#if file.streamStat}
        <StreamInfo
          streamInfo={formatStreamInfo(file.streamStat, fileName)}
          isError={!!file.streamStat?.error}>
        </StreamInfo>
      {/if}
      <button
        class="download-btn url-action-btn uk-button uk-button-link  uk-margin-small-left"
        on:click|preventDefault={confirm(() => downloadFile())}
      >
        Download
        <i class="uk-icon" uk-icon="icon: cloud-download; ratio: 0.8" />&nbsp;
      </button>

    </div>
    <span slot="title"
    >Download file <code>{fileName}</code></span
    >
    <span slot="description"
    >Current file fill be removed and download process will be started</span
    >
    <span slot="confirm">Start download</span>
  </Confirm>
</template>

<style lang='stylus'>
  .file-name
    color: var(--primary-text-color)
    padding-right: 6px

  .file-info-container
    &:hover
      .download-btn
        opacity: 1

  .download-btn
    opacity: 0

  .uk-progress
    height: 3px;
    margin-bottom: 0
    margin-top: 0
    background-color: #fff

  progress::-webkit-progress-value {
    background: var(--warning-color);
  }

  progress::-moz-progress-bar {
    background: var(--warning-color);
  }

  .download-percents {
    font-size: smaller
    margin: 0 4px
  }


</style>
