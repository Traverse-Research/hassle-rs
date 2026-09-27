// Library: imports add4 from the other library and provides a compute entry point.
float4 add4(float4 a, float4 b);

RWStructuredBuffer<float4> output : register(u0);

[shader("compute")]
[numthreads(1, 1, 1)]
void main()
{
    output[0] = add4(float4(1.0, 2.0, 3.0, 4.0), float4(10.0, 20.0, 30.0, 40.0));
}
