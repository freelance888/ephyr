<script lang="js">
  import Confirm from '../common/Confirm.svelte';
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
    ONLINE,
  } from '../../utils/constants';

  import {
    MoveInputInDirection,
    SingleFile,
  } from '../../../api/client.graphql';
  import { showError } from '../../utils/util';
  import { mutation, subscribe } from 'svelte-apollo';

  const moveInputInDirectionMutation = mutation(MoveInputInDirection);

  export let endpoint;
  export let input;
  export let input_url;
  export let restream_id;

  export let with_label;
  export let show_controls;
  export let show_move_up;
  export let show_move_down;
  export let show_up_confirmation;

  const backupFile = subscribe(SingleFile, {
    variables: { id: endpoint.fileId },
    errorPolicy: 'all',
  });

  $: isPull = !!input.src && input.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!input.src && input.src.__typename === 'FailoverInputSrc';

  $: currentFile = $backupFile?.data?.file;
  $: isFile = endpoint.kind === ENDPOINT_KIND_FILE;

  $: isFileError = currentFile?.state === FILE_DOWNLOAD_ERROR;

  $: alertDanger = isFile
    ? isFileError || !input.enabled
    : endpoint.status === OFFLINE;

  $: alertWarning = isFile
    ? currentFile?.state === FILE_PENDING ||
      currentFile?.state === FILE_DOWNLOADING
    : endpoint.status === INITIALIZING;

  $: alertSuccess = isFile
    ? currentFile?.state === FILE_LOCAL
    : endpoint.status === ONLINE;

  async function moveUp() {
    try {
      await moveInputInDirectionMutation({
        variables: {
          restream_id: restream_id,
          input_id: input.id,
          direction: 'UP',
        },
      });
    } catch (e) {
      showError(e.message);
    }
  }

  async function moveDown() {
    try {
      await moveInputInDirectionMutation({
        variables: {
          restream_id: restream_id,
          input_id: input.id,
          direction: 'DOWN',
        },
      });
    } catch (e) {
      showError(e.message);
    }
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
            class="fas fa-arrow-right"
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
      <FileInfo file={currentFile} showDownloadLink={true} />
    {:else}
      <Url
        streamInfo={formatStreamInfo(endpoint.streamStat)}
        isError={!!endpoint.streamStat?.error}
        url={input_url}
      />

      {#if !isFailover}
        <!-- Do not display UP button on endpoint at the 1st position. Display confirm dialog for endpoint at the second position -->
        {#if show_move_up}
          {#if show_up_confirmation}
            <Confirm let:confirm>
              <button
                class="uk-button-default arrows"
                data-testid="move-input-up"
                title="Move up"
                on:click={() => confirm(moveUp)}
                ><span>↑</span>
              </button>
              <span slot="title">Move up</span>
              <span slot="description"
                >Move this endpoint up and replace primary endpoint.
              </span>
              <span slot="confirm">Move up</span>
            </Confirm>
          {:else}
            <button
              class="uk-button-default arrows"
              data-testid="move-input-up"
              title="Move up"
              on:click={moveUp}
              ><span>↑</span>
            </button>
          {/if}
        {:else}
          <button
            style="border:none"
            class="uk-button-default arrows"
            data-testid="move-input-up"
            title=""
            ><span>&nbsp&nbsp</span>
          </button>
        {/if}

        <!-- Do not display DOWN button on endpoint on the last position. Display confirm dialog for endpoint at the first position -->
        {#if show_move_down}
          {#if !show_move_up}
            <Confirm let:confirm>
              <button
                class="uk-button-default arrows"
                data-testid="move-input-down"
                title="Move down"
                on:click={() => confirm(moveDown)}
                ><span>↓</span>
              </button>
              <span slot="title">Move down</span>
              <span slot="description"
                >Move this endpoint down. Note, this endpoint is primary, it
                will be replaced by the following endpoint.
              </span>
              <span slot="confirm">Move down</span>
            </Confirm>
          {:else}
            <button
              class="uk-button-default arrows"
              data-testid="move-input-down"
              title="Move down"
              on:click={moveDown}
              ><span>↓</span>
            </button>
          {/if}
        {:else}
          <button
            style="border:none"
            class="uk-button-default arrows"
            data-testid="move-input-down"
            title=""
            ><span>&nbsp&nbsp</span>
          </button>
        {/if}
      {/if}

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

    .arrows
      width: 22px
</style>
