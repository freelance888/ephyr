<script lang="js">
  import { mutation } from 'svelte-apollo';

  import { SetEndpointLabel } from '../../../api/client.graphql';
  import { saveOrCloseByKeys } from '../../utils/directives.util';

  import { showError } from '../../utils/util';

  const setLabelMutation = mutation(SetEndpointLabel);

  export let endpoint;
  export let input;
  export let restream_id;
  export let show_controls;

  let label_component;
  let label_input;
  let show_edit = false;

  $: btn_text = endpoint.label ? 'Edit label' : 'Add label';

  async function showEdit() {
    show_edit = true;
  }

  async function cancelEdit() {
    label_component.value = endpoint.label;
    show_edit = false;
  }

  async function submit() {
    const variables = {
      restream_id: restream_id,
      input_id: input.id,
      endpoint_id: endpoint.id,
      label: label_input.value,
    };
    try {
      let result_val = await setLabelMutation({ variables });
      if (result_val.data.setEndpointLabel) {
        endpoint.label = label_input.value;
        label_component.value = endpoint.label;
        show_edit = false;
      } else if (result_val.data.setEndpointLabel === null) {
        showError('No given input endpoint.');
      }
    } catch (e) {
      showError(e.message);
    }
  }

  function init_input(label_input) {
    label_input.value = endpoint.label;
    label_input.focus();
  }
</script>

<template>
  <div class="endpoint-label">
    <span
      data-testid="endpoint-label-text"
      class={endpoint.label ? 'uk-margin-small-left' : ''}
      bind:this={label_component}
      class:hidden={show_edit}>{endpoint.label ? endpoint.label : ''}</span
    >
    {#if show_edit}
      <input
        type="text"
        bind:this={label_input}
        use:init_input
        use:saveOrCloseByKeys={{
          save: submit,
          close: cancelEdit,
        }}
        on:focusout|preventDefault={() => {
          cancelEdit();
        }}
      />
    {/if}
    <button
      type="button"
      class="edit-label-btn uk-button uk-button-link uk-button-small"
      class:hidden={!show_controls}
      on:click|preventDefault={() => {
        showEdit();
      }}
    >
      <span class="uk-margin-small-left">{btn_text}</span>
      <i class="far fa-edit" />
    </button>
  </div>
</template>

<style lang="stylus">
  .endpoint-label
    display: inline-flex
    color: #999

    .hidden
      display: none

    .edit-label-btn
      align-self: flex-start
      text-transform: initial
      text-decoration: none
      font-size: 13px
      transition: 0.1s ease-in
</style>
