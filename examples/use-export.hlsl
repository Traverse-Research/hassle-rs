RWTexture2D<float4> g_output : register(u0, space0);

int valueTwo();

[numthreads(8, 8, 1)]
[shader("compute")]
void copyCs(uint3 dispatchThreadId : SV_DispatchThreadID)
{
	g_output[dispatchThreadId.xy] = valueTwo();
}