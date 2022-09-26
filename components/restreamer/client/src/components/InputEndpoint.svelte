<script lang="js">
  import { mutation } from 'svelte-apollo';

  import { SetEndpointLabel } from '../../api/client.graphql';

  import Url from './common/Url.svelte';
  import { showError } from '../utils/util';

  const changeLabelMutation = mutation(SetEndpointLabel);

  export let endpoint;
  export let input;
  export let input_url;
  export let restream_id;
  export let with_label;

  let label_component;
  let label_input;
  let editing_label = false;

  $: isPull = !!input.src && input.src.__typename === 'RemoteInputSrc';
  $: isFailover = !!input.src && input.src.__typename === 'FailoverInputSrc';

  async function editLabel(startEdit) {
    if (startEdit) {
      editing_label = true;
    } else {
      const variables = {
        restream_id: restream_id,
        input_id: input.id,
        endpoint_id: endpoint.id,
        label: label_input.value,
      };
      try {
        let result_val = await changeLabelMutation({ variables });
        if (result_val.data.setEndpointLabel) {
          endpoint.label = label_input.value;
          label_component.value = endpoint.label;
          editing_label = false;
        } else {
          showError('Provided text has invalid characters or is too long.');
        }
      } catch (e) {
        showError(e.message);
      }
    }
  }

  function init_input(label_input) {
    label_input.value = endpoint.label;
    label_input.focus();
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

    <Url url={input_url} />
    {#if with_label}
      <div class="endpoint-label">
        <span bind:this={label_component} class:hidden={editing_label}
          >{endpoint.label ? endpoint.label : ''}</span
        >
        {#if editing_label}
          <input
            bind:this={label_input}
            use:init_input
            on:focusout|preventDefault={() => {
              editLabel(false);
            }}
          />
        {/if}
        <a
          class="edit-label"
          href="/"
          on:click|preventDefault={() => {
            editLabel(true);
          }}
        >
          <i class="far fa-edit" title="Edit label" />
        </a>
      </div>
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

    .endpoint-label
      margin-left 5px
      color: #999

      &:hover
        .edit-label
          opacity: 1

      .hidden
        display: none

      .edit-label
        opacity: 0
        transition: opacity .3s ease
        color: #666
        outline: none
        &:hover
          opacity: 1
          text-decoration: none
          color: #444

    .endpoint-status-icon
      flex-shrink: 0
      margin-right: 5px
</style>
