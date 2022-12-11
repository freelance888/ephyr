<script lang="js">
  import Url from '../common/Url.svelte';
  import InputEndpointLabel from './InputEndpointLabel.svelte';

  export let endpoint;
  export let input;
  export let input_url;
  export let restream_id;
  export let with_label;
  export let show_controls;

  $: isPull = !!input.src && input.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!input.src && input.src.__typename === 'FailoverInputSrc';

  const formatStreamInfo = (streamStat) => {
    return streamStat ? `
        <span><strong>${input.key}</strong></span>
        <br/>
        <span><strong>video</strong>&#58; ${streamStat.videoCodecName}, </span>
        <span>${streamStat.videoWidth}x${streamStat.videoHeight},</span>
        <span>${streamStat.videoRFrameRate?.replace('/1', '')} FPS</span>
        <br/>
        <span><strong>audio</strong>&#58; ${streamStat.audioCodecName},</span>
        <span>${streamStat.audioSampleRate},</span>
        <span>${streamStat.audioChannelLayout},</span>
        <span>channels&#58; ${streamStat.audioChannels}</span>
` : '';
  }

</script>

<template>
  <div class="endpoint">
    <div
      class:endpoint-status-icon={true}
      data-testid={`endpoint-status:${endpoint.status}`}
      class:uk-alert-danger={endpoint.status === 'OFFLINE'}
      class:uk-alert-warning={endpoint.status === 'INITIALIZING'}
      class:uk-alert-success={endpoint.status === 'ONLINE'}
    >
      {#if isFailover || endpoint.kind !== 'RTMP'}
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

    <Url url={input_url} streamInfo={formatStreamInfo(endpoint.streamStat)} />
    {#if with_label}
      <InputEndpointLabel {endpoint} {restream_id} {input} {show_controls} />
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
