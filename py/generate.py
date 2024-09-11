from typing import AsyncGenerator
from vllm import LLM, SamplingParams

from vllm.engine.arg_utils import AsyncEngineArgs
from vllm.engine.async_llm_engine import AsyncLLMEngine


async def async_initialize() -> LLM:
    args = AsyncEngineArgs(model="microsoft/phi-1_5")
    return AsyncLLMEngine(args)


async def async_generate(
    llm: AsyncLLMEngine, sp: SamplingParams
) -> AsyncGenerator[str, None]:
    question = "Tell me a joke.\n"
    prompt = f"<|user|>\n{question}<|end|>\n<|assistant|>\n"
    inputs = {"prompt": prompt}
    generator = llm.generate(inputs, sampling_params=sampling_params)
    async for request_output in results_generator:
        prompt = request_output.prompt
        text_outputs = [prompt + output.text for output in request_output.outputs]
        ret = {"text": text_outputs}
        yield json.dumps(ret)


def initialize() -> LLM:
    llm = LLM(model="microsoft/phi-1_5", download_dir='/files/huggingface-hub/')
    return llm


def get_sampling_params() -> SamplingParams:
    sampling_params = SamplingParams(
        temperature=0.2, max_tokens=64, stop_token_ids=None
    )
    return sampling_params


def generate(llm: LLM, sp: SamplingParams):
    question = "Tell me a joke.\n"
    prompt = f"<|user|>\n{question}<|end|>\n<|assistant|>\n"
    inputs = {"prompt": prompt}
    outputs = llm.generate(inputs, sampling_params=sp)
    return outputs[0].prompt + outputs[0].outputs[0].text


# llm = initialize()
# sp = get_sampling_params()
# print(generate(llm, sp))
