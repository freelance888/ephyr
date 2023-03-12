<script lang="js">
  import Url from '../common/Url.svelte';
  import InputEndpointLabel from './InputEndpointLabel.svelte';
  import FileInfo from '../common/FileInfo.svelte';
  import { formatStreamInfo } from '../../utils/streamInfo.util';
  import {
    ENDPOINT_KIND_FILE,
    ENDPOINT_KIND_RTMP,
    FILE_DOWNLOAD_ERROR,
    FILE_DOWNLOADING,
    FILE_LOCAL,
    FILE_PENDING,
    INITIALIZING,
    OFFLINE,
    ONLINE
  } from '../../utils/constants';

  export let endpoint;
  export let input;
  export let input_url;
  export let restream_id;
  export let with_label;
  export let show_controls;
  export let files;

  $: isPull = !!input.src && input.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!input.src && input.src.__typename === 'FailoverInputSrc';

  $: currentFile = searchFile(files);
  $: isFile = endpoint.kind === ENDPOINT_KIND_FILE;

  $: isFileError = currentFile?.state === FILE_DOWNLOAD_ERROR;

  $: alertDanger = isFile ? isFileError : endpoint.status === OFFLINE;

  $: alertWarning = isFile
    ? currentFile?.state === FILE_PENDING || currentFile?.state === FILE_DOWNLOADING
    : endpoint.status === INITIALIZING;

  $: alertSuccess = isFile
    ? currentFile?.state === FILE_LOCAL
    : endpoint.status === ONLINE;

  const searchFile = (allFiles) => {
    return allFiles
      ? allFiles.find((val) => val.fileId === endpoint.fileId)
      : undefined;
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
      {#if isFile}
        <span
          ><i
            class="fas fa-file"
            title="Serves live {endpoint.kind} stream"
          /></span
        >
      {:else if isFailover || endpoint.kind !== ENDPOINT_KIND_RTMP}
        {#if endpoint.status === ONLINE}
          <span
            ><i
              class="fas fa-circle"
              title="Serves {isFailover
                ? 'failover '
                : ''}live {endpoint.kind} stream"
            /></span
          >
        {:else if endpoint.status === INITIALIZING}
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

    {#if isFile && currentFile}
      <FileInfo file={currentFile}></FileInfo>
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
