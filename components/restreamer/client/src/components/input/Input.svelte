<script lang="js">
  import { mutation, subscribe } from 'svelte-apollo';

  import { DisableInput, EnableInput, Info } from '../../../api/client.graphql';

  import { showError } from '../../utils/util';

  import Toggle from '../common/Toggle.svelte';
  import Confirm from '../common/Confirm.svelte';
  import InputEndpoint from './InputEndpoint.svelte';

  const disableInputMutation = mutation(DisableInput);
  const enableInputMutation = mutation(EnableInput);

  const info = subscribe(Info, { errorPolicy: 'all' });

  export let public_host = 'localhost';
  export let restream_id;
  export let restream_key;
  export let value;
  export let with_label;
  export let show_controls;
  export let show_move_up = false;
  export let show_move_down = false;
  export let show_up_confirmation = true;

  $: isPull = !!value.src && value.src.__typename === 'RemoteInputSrc';

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
      return `http://${public_host}:8000/${restream_key}/${value.key}.m3u8?vHost=hls`;
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
        <InputEndpoint
          {endpoint}
          input={value}
          input_url={getInputUrl(endpoint)}
          {restream_id}
          {with_label}
          {show_controls}
          {show_move_up}
          {show_up_confirmation}
          {show_move_down}
        />
      {/each}
    </div>
  </div>
</template>

<style lang="stylus">
  .input
    display: flex;
    align-items: baseline;

  .endpoints
    margin-left: 4px
</style>
