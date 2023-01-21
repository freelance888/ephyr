<script lang="js">
  import Url from '../common/Url.svelte';
  import InputEndpointLabel from './InputEndpointLabel.svelte';
  import Confirm from '../common/Confirm.svelte';

  export let endpoint;
  export let input;
  export let input_url;
  export let restream_id;
  export let with_label;
  export let show_controls;
  export let files;

  $: isPull = !!input.src && input.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!input.src && input.src.__typename === 'FailoverInputSrc';

  $: current_file = searchFile($files.data);
  $: isFile = endpoint.kind === 'FILE';

  $: alertDanger = isFile
    ? current_file?.state === 'ERROR'
    : endpoint.status === 'OFFLINE';

  $: alertWarning = isFile
    ? (current_file?.state === 'PENDING' || current_file?.state === 'DOWNLOADING')
    : endpoint.status === 'INITIALIZING';

  $: alertSuccess = isFile
    ? current_file?.state === 'LOCAL'
    : endpoint.status === 'ONLINE';

  $: fileDownloadProgress = getFileDownloadProgress(current_file);

  function searchFile(all_files) {
    return all_files?.files
      ? all_files.files.find((val) => val.fileId === endpoint.fileId)
      : undefined;
  }

  const formatStreamInfo = (streamStat) => {
    if (streamStat) {
      return streamStat.error
        ? streamStat.error
        : `<span><strong>${input.key}</strong></span>
          <br/>
          <span><strong>video</strong>&#58; ${
            streamStat.videoCodecName
          }, </span>
          <span>${streamStat.videoWidth}x${streamStat.videoHeight},</span>
          <span>${streamStat.videoRFrameRate?.replace('/1', '')} FPS</span>
          <br/>
          <span><strong>audio</strong>&#58; ${streamStat.audioCodecName},</span>
          <span>${streamStat.audioSampleRate},</span>
          <span>${streamStat.audioChannelLayout},</span>
          <span>channels&#58; ${streamStat.audioChannels}</span>`;
    }

    return '';
  };

  const getFileName = (current_file) => current_file.name ? current_file.name : current_file.fileId;

  const getFileDownloadProgress = (current_file) => {
    let value = current_file?.downloadState &&
      current_file.downloadState.currentProgress !==
      current_file.downloadState.maxProgress
        ? (
            (current_file.downloadState.currentProgress /
              current_file.downloadState.maxProgress) *
            100
          )
        : 0

    return value < 0 || value >= 100 ? undefined : value;
  }

</script>

<template>
  <div class="endpoint">
    <div
      class:endpoint-status-icon={true}
      data-testid={`endpoint-status:${endpoint.status}`}
      class:uk-alert-danger={alertDanger}
      class:uk-alert-warning={alertWarning}
      class:uk-alert-success={alertSuccess}
    >
      {#if isFile}
        <span
        ><i
          class="fas fa-file"
          title="Serves live {endpoint.kind} stream"
        /></span
        >
      {:else if isFailover || endpoint.kind !== 'RTMP'}
        {#if endpoint.status === 'ONLINE'}
          <span
            ><i
              class="fas fa-circle"
              title="Serves {isFailover
                ? 'failover '
                : ''}live {endpoint.kind} stream"
            /></span
          >
        {:else if endpoint.status === 'INITIALIZING'}
          <span
            ><i
              class="fas fa-dot-circle"
              title="Serves {isFailover
                ? 'failover '
                : ''}live {endpoint.kind} stream"
            /></span
          >
        {:else}
          <span
            ><i
              class="far fa-dot-circle"
              title="Serves {isFailover
                ? 'failover '
                : ''}live {endpoint.kind} stream"
            /></span
          >
        {/if}
      {:else if isPull}
        <span
          ><i
            class="fas fa-arrow-down"
            title="Pulls {input.key} live {endpoint.kind} stream"
          />
        </span>
      {:else}
        <span
          ><i
            class="fas fa-arrow-right"
            title="Accepts {input.key} live {endpoint.kind} stream"
          />
        </span>
      {/if}
    </div>

    {#if isFile && current_file}
      <Confirm let:confirm>
          <div class='uk-flex uk-flex-middle'>
            <div class='uk-flex uk-flex-column'>
              <a href="/" class='file-name' on:click|preventDefault={confirm(() => alert('123'))}>
                { getFileName(current_file) }
              </a>
              <div class='uk-flex uk-flex-middle'>
                {#if fileDownloadProgress}
                  <progress class="uk-progress" value="{fileDownloadProgress}" max="100"></progress>
                  <span class='uk-display-inline-block download-percents'>{ fileDownloadProgress.toFixed(0) }</span>%
                {/if}
              </div>
            </div>
            <Url
              streamInfo={formatStreamInfo(endpoint.streamStat)}
              isError={!!endpoint.streamStat?.error}
            />
          </div>
        <span slot="title">Download file <code>{getFileName(current_file)}</code></span>
        <span slot="description">Current file fill be removed and download process will be started</span>
        <span slot="confirm">Start download</span>
      </Confirm>

    {:else}
      <Url
        streamInfo={formatStreamInfo(endpoint.streamStat)}
        isError={!!endpoint.streamStat?.error}
        url={input_url}
      />
        {#if with_label}
          <InputEndpointLabel {endpoint} {restream_id} {input} {show_controls} />
        {/if}
    {/if}
  </div>

</template>

<style lang="stylus">
  .endpoint
    display: flex

    .fa-arrow-down, .fa-arrow-right
      font-size: 14px
      cursor: help

    .fa-circle, .fa-dot-circle
      font-size: 13px
      cursor: help

    .endpoint-status-icon
      flex-shrink: 0
      margin-right: 5px

    .file-name
      color: var(--primary-text-color)

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
