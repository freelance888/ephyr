<script lang="js">
  import { mutation, subscribe } from 'svelte-apollo';

  import {
    DisableInput,
    EnableInput,
    Info,
  } from '../../api/client.graphql';

  import { showError } from '../utils/util';

  import Toggle from './common/Toggle.svelte';
  import Confirm from './common/Confirm.svelte';
  import Url from './common/Url.svelte';

  const disableInputMutation = mutation(DisableInput);
  const enableInputMutation = mutation(EnableInput);

  const info = subscribe(Info, { errorPolicy: 'all' });

  export let public_host = 'localhost';
  export let restream_id;
  export let restream_key;
  export let value;

  $: isPull = !!value.src && value.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!value.src && value.src.__typename === 'FailoverInputSrc';

  $: toggleStatusText = value.enabled ? 'Disable' : 'Enable';

  $: enableConfirmation = $info.data
    ? $info.data.info.enableConfirmation
    : true;

  async function toggle() {
    const variables = { restream_id, input_id: value.id };
    try {
      if (value.enabled) {
        await disableInputMutation({ variables });
      } else {
        await enableInputMutation({ variables });
      }
    } catch (e) {
      showError(e.message);
    }
  }

  function getInputUrl(endpoint) {
    if (endpoint.kind === 'HLS')
      return `http://${public_host}:8000/hls/${restream_key}/${value.key}.m3u8`;
    else if (isPull) return value.src.url;
    else return `rtmp://${public_host}/${restream_key}/${value.key}`;
  }
</script>

<template>
  <div class="input">
    <Confirm let:confirm>
      <Toggle
        id="input-toggle-{value.id}"
        checked={value.enabled}
        confirmFn={enableConfirmation ? confirm : undefined}
        onChangeFn={toggle}
      />
      <span slot="title"
        >{toggleStatusText} <code>{restream_key}</code> input</span
      >
      <span slot="description">Are you sure about it?</span>
      <span slot="confirm">{toggleStatusText}</span>
    </Confirm>
    <div class="endpoints">
      {#each value.endpoints as endpoint}
        <div class="endpoint">
          <div
            class:endpoint-status-icon={true}
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
                      : ''} live {endpoint.kind} stream"
                  /></span
                >
              {:else}
                <span
                  ><i
                    class="far fa-dot-circle"
                    title="Serves {isFailover
                      ? 'failover '
                      : ''} live {endpoint.kind} stream"
                  /></span
                >
              {/if}
            {:else if isPull}
              <span
                ><i
                  class="fas fa-arrow-down"
                  title="Pulls {value.key} live {endpoint.kind} stream"
                />
              </span>
            {:else}
              <span
                ><i
                  class="fas fa-arrow-right"
                  title="Accepts {value.key} live {endpoint.kind} stream"
                />
              </span>
            {/if}
          </div>

          <Url url={getInputUrl(endpoint)} />
        </div>
      {/each}
    </div>
  </div>
</template>

<style lang="stylus">
  .fa-arrow-down, .fa-arrow-right
    font-size: 14px
    cursor: help

  .fa-circle, .fa-dot-circle
    font-size: 13px
    cursor: help

  .input
    display: flex;
    align-items: baseline;

  .endpoints
    margin-left: 4px

  .endpoint
    display: flex

  .endpoint .endpoint-status-icon
    flex-shrink: 0
    margin-right: 5px
</style>
