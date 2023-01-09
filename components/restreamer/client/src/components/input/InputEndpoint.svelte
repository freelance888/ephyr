<script lang="js">
  import Url from '../common/Url.svelte';
  import InputEndpointLabel from './InputEndpointLabel.svelte';

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
    ? current_file && current_file.state === 'ERROR'
    : endpoint.status === 'OFFLINE';
  $: alertWarning = isFile
    ? current_file &&
    (current_file.state === 'PENDING' || current_file.state === 'DOWNLOADING')
    : endpoint.status === 'INITIALIZING';
  $: alertSuccess = isFile
    ? current_file && current_file.state === 'LOCAL'
    : endpoint.status === 'ONLINE';

  function searchFile(all_files) {
    if (all_files && all_files.files) {
      return all_files.files.find((val) => val.fileId === endpoint.fileId);
    } else {
      return undefined;
    }
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
      {#if endpoint.kind === 'FILE'}
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

    {#if endpoint.kind === 'FILE' && current_file}
      <Url
        streamInfo={formatStreamInfo(endpoint.streamStat)}
        isError={!!endpoint.streamStat?.error}
        url="{current_file.name ? current_file.name : current_file.fileId}
      {current_file.downloadState &&
        current_file.downloadState.currentProgress !==
          current_file.downloadState.maxProgress
          ? (
              (current_file.downloadState.currentProgress /
                current_file.downloadState.maxProgress) *
              100
            ).toFixed(2) + '%'
          : ''}"
      />
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
</style>
